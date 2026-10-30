use oxc_ast::CommentKind;
use oxc_data_structures::branch_hints::cold_path;
use oxc_syntax::line_terminator::is_line_terminator;

use crate::{config::LexerConfig as Config, diagnostics};

use super::{
    Kind, Lexer,
    search::{
        SEARCH_BATCH_SIZE, SafeByteMatchTable, byte_search, match_star_slash_and_at,
        safe_byte_match_table,
    },
    source::SourcePosition,
};

// Irregular line breaks - '\u{2028}' (LS) and '\u{2029}' (PS)
const LS_OR_PS_FIRST: u8 = 0xE2;
const LS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA8];
const PS_BYTES_2_AND_3: [u8; 2] = [0x80, 0xA9];

static LINE_BREAK_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'\r' | b'\n' | LS_OR_PS_FIRST));

static MULTILINE_COMMENT_START_TABLE: SafeByteMatchTable =
    safe_byte_match_table!(|b| matches!(b, b'*' | b'\r' | b'\n' | LS_OR_PS_FIRST));

impl<'a, C: Config> Lexer<'a, C> {
    /// Section 12.4 Single Line Comment
    pub(super) fn skip_single_line_comment(&mut self, kind: CommentKind) -> Kind {
        byte_search! {
            lexer: self,
            table: LINE_BREAK_TABLE,
            continue_if: (next_byte, pos) {
                // Match found. Decide whether to continue searching.
                // If this is end of comment, create trivia, and advance `pos` to after line break.
                // Do that here rather than in `handle_match`, to avoid branching twice on value of
                // the matched byte.
                #[expect(clippy::if_not_else)]
                if next_byte != LS_OR_PS_FIRST {
                    // `\r` or `\n`
                    self.trivia_builder
                        .add_line_comment(self.token.start(), self.source.offset_of(pos), kind, self.source.whole());
                    // SAFETY: Safe to consume `\r` or `\n` as both are ASCII
                    pos = unsafe { pos.add(1) };
                    // We've found the end. Do not continue searching.
                    false
                } else {
                    // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
                    // Either way, Unicode is uncommon, so make this a cold branch.
                    cold_path();
                    // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                    // So safe to advance `pos` by 1 and read 2 bytes.
                    let next2 = unsafe { pos.add(1).read2() };
                    if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                        // Irregular line break
                        self.trivia_builder
                            .add_line_comment(self.token.start(), self.source.offset_of(pos), kind, self.source.whole());
                        // Advance `pos` to after this char.
                        // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                        // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                        pos = unsafe { pos.add(3) };
                        // We've found the end. Do not continue searching.
                        false
                    } else {
                        // Some other Unicode char beginning with `0xE2`.
                        // Skip 3 bytes (macro skips 1 already, so skip 2 here), and continue searching.
                        // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                        // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                        pos = unsafe { pos.add(2) };
                        true
                    }
                }
            },
            handle_eof: {
                self.trivia_builder.add_line_comment(self.token.start(), self.offset(), kind, self.source.whole());
                return Kind::Skip;
            },
        };

        self.token.set_is_on_new_line(true);
        Kind::Skip
    }

    /// Section 12.4 Multi Line Comment
    pub(super) fn skip_multi_line_comment(&mut self) -> Kind {
        // We need to identify if comment contains line breaks or not
        // (`CommentKind::SingleLineBlock` or `CommentKind::MultiLineBlock`).
        // So we have to use the loop below for the first line of the comment even if
        // `Token`'s `is_on_new_line` flag is already set.
        // If the loop finds a line break before end of the comment, we then switch to
        // the faster `skip_multi_line_comment_after_line_break` which searches
        // for the end of the comment using `memchr`.

        byte_search! {
            lexer: self,
            table: MULTILINE_COMMENT_START_TABLE,
            continue_if: (next_byte, pos) {
                // Match found. Decide whether to continue searching.
                if next_byte == b'*' {
                    // SAFETY: Next byte is `*` (ASCII) so after it is UTF-8 char boundary
                    let after_star = unsafe { pos.add(1) };
                    if after_star.is_not_end_of(&self.source) {
                        // If next byte isn't `/`, continue
                        // SAFETY: Have checked there's at least 1 further byte to read
                        if unsafe { after_star.read() } == b'/' {
                            // Consume `*/`
                            // SAFETY: Consuming `*/` leaves `pos` on a UTF-8 char boundary
                            pos = unsafe { pos.add(2) };
                            false
                        } else {
                            true
                        }
                    } else {
                        // This is last byte in file. Continue to `handle_eof`.
                        // This is illegal in valid JS, so mark this branch cold.
                        cold_path();
                        true
                    }
                } else if next_byte == LS_OR_PS_FIRST {
                    // `0xE2`. Could be first byte of LS/PS, or could be some other Unicode char.
                    // Either way, Unicode is uncommon, so make this a cold branch.
                    cold_path();
                    // SAFETY: Next byte is `0xE2` which is always 1st byte of a 3-byte UTF-8 char.
                    // So safe to advance `pos` by 1 and read 2 bytes.
                    let next2 = unsafe { pos.add(1).read2() };
                    if matches!(next2, LS_BYTES_2_AND_3 | PS_BYTES_2_AND_3) {
                        // Irregular line break
                        self.token.set_is_on_new_line(true);
                        // SAFETY: Consuming this 3-byte UTF-8 char leaves a UTF-8 char boundary.
                        let after_line_break = unsafe { pos.add(3) };
                        return self.skip_multi_line_comment_after_line_break(after_line_break);
                    }
                    // Some other Unicode char beginning with `0xE2`.
                    // Skip 3 bytes (macro skips 1 already, so skip 2 here), and continue searching.
                    // SAFETY: `0xE2` is always 1st byte of a 3-byte UTF-8 char,
                    // so consuming 3 bytes will place `pos` on next UTF-8 char boundary.
                    pos = unsafe { pos.add(2) };
                    true
                } else {
                    // Regular line break.
                    // No need to look for more line breaks, so switch to faster search just for `*/`.
                    self.token.set_is_on_new_line(true);
                    // SAFETY: Regular line breaks are ASCII, so skipping 1 byte is a UTF-8 char boundary.
                    let after_line_break = unsafe { pos.add(1) };
                    return self.skip_multi_line_comment_after_line_break(after_line_break);
                }
            },
            handle_eof: {
                self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
                return Kind::Eof;
            },
        };

        self.trivia_builder.add_block_comment(
            self.token.start(),
            self.offset(),
            CommentKind::SingleLineBlock,
            self.source.whole(),
        );
        Kind::Skip
    }

    fn skip_multi_line_comment_after_line_break(&mut self, pos: SourcePosition<'a>) -> Kind {
        let remaining = self.source.str_from_pos_to_end(pos).as_bytes();
        // Check if `@license` or `@preserve` occurred before the first line break.
        let comment_content_start = (self.token.start() + 2) as usize;
        let first_line_end = self.source.offset_of(pos) as usize;
        let mut has_license = check_license_in_slice(
            &self.source.whole().as_bytes()[comment_content_start..first_line_end],
        );

        let mut offset = 0usize;
        while offset + SEARCH_BATCH_SIZE < remaining.len() {
            // SAFETY: `offset + 16 < remaining.len()`, so at least 17 bytes are readable at `ptr`.
            let (end_lane, at_bits) =
                unsafe { match_star_slash_and_at(remaining.as_ptr().add(offset)) };
            if let Some(lane) = end_lane {
                if !has_license && at_bits != 0 {
                    has_license = check_at_bits(remaining, offset, lane, at_bits);
                }
                // SAFETY: `pos + offset + lane + 2` is end of `*/`, so a valid `SourcePosition`.
                self.source.set_position(unsafe { pos.add(offset + lane + 2) });
                self.trivia_builder.add_block_comment_with_license_hint(
                    self.token.start(),
                    self.offset(),
                    CommentKind::MultiLineBlock,
                    self.source.whole(),
                    has_license,
                );
                return Kind::Skip;
            }
            if !has_license && at_bits != 0 {
                has_license = check_at_bits(remaining, offset, SEARCH_BATCH_SIZE, at_bits);
            }
            offset += SEARCH_BATCH_SIZE;
        }
        while offset + 1 < remaining.len() {
            let b = remaining[offset];
            if b == b'*' && remaining[offset + 1] == b'/' {
                // SAFETY: `pos + offset + 2` is end of `*/`, so a valid `SourcePosition`.
                self.source.set_position(unsafe { pos.add(offset + 2) });
                self.trivia_builder.add_block_comment_with_license_hint(
                    self.token.start(),
                    self.offset(),
                    CommentKind::MultiLineBlock,
                    self.source.whole(),
                    has_license,
                );
                return Kind::Skip;
            }
            if !has_license && b == b'@' && matches_license_or_preserve_at(remaining, offset) {
                has_license = true;
            }
            offset += 1;
        }

        self.source.advance_to_end();
        self.error(diagnostics::unterminated_multi_line_comment(self.unterminated_range()));
        Kind::Eof
    }

    /// Section 12.5 Hashbang Comments.
    ///
    /// # SAFETY
    /// Next 2 bytes must be `#!`.
    pub(super) unsafe fn read_hashbang_comment(&mut self) -> Kind {
        debug_assert_eq!(self.peek_2_bytes(), Some([b'#', b'!']));

        // SAFETY: Caller guarantees next 2 bytes are `#!`
        unsafe {
            self.source.next_byte_unchecked();
            self.source.next_byte_unchecked();
        }

        while let Some(c) = self.peek_char() {
            if is_line_terminator(c) {
                break;
            }
            self.consume_char();
        }
        Kind::HashbangComment
    }
}

#[inline]
fn matches_license_or_preserve_at(bytes: &[u8], i: usize) -> bool {
    const LICENSE_LEN: usize = b"@license".len();
    const PRESERVE_LEN: usize = b"@preserve".len();
    if bytes.len() - i < LICENSE_LEN {
        return false;
    }
    // SAFETY: `bytes.len() - i >= 8`, and `bytes.len() - i >= 9` is checked before reading byte 8.
    unsafe {
        let ptr = bytes.as_ptr().add(i);
        let word = ptr.cast::<u64>().read_unaligned();
        word == u64::from_ne_bytes(*b"@license")
            || (word == u64::from_ne_bytes(*b"@preserv")
                && bytes.len() - i >= PRESERVE_LEN
                && *ptr.add(8) == b'e')
    }
}

#[inline]
fn check_license_in_slice(slice: &[u8]) -> bool {
    for i in memchr::memchr_iter(b'@', slice) {
        if matches_license_or_preserve_at(slice, i) {
            return true;
        }
    }
    false
}

#[inline]
fn check_at_bits(
    remaining: &[u8],
    base_offset: usize,
    limit_lane: usize,
    mut at_bits: u64,
) -> bool {
    if cfg!(all(target_arch = "aarch64", target_endian = "big")) {
        // Normalize the four-bit lanes to the little-endian mask layout.
        at_bits = at_bits.reverse_bits();
    }
    while at_bits != 0 {
        let lane = if cfg!(target_arch = "aarch64") {
            (at_bits.trailing_zeros() >> 2) as usize
        } else {
            at_bits.trailing_zeros() as usize
        };
        if lane >= limit_lane {
            break;
        }
        // Check against `remaining` (which includes characters up to `*/`, and `*/` cannot match
        // `icense` or `preserve`).
        if matches_license_or_preserve_at(remaining, base_offset + lane) {
            return true;
        }
        if cfg!(target_arch = "aarch64") {
            at_bits &= !(0xFu64 << (lane * 4));
        } else {
            at_bits &= at_bits - 1;
        }
    }
    false
}

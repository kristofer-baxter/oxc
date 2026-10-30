//! An Ecma-262 Lexer / Tokenizer
//! Prior Arts:
//!     * [jsparagus](https://github.com/mozilla-spidermonkey/jsparagus/blob/24004745a8ed4939fc0dc7332bfd1268ac52285f/crates/parser/src)
//!     * [rome](https://github.com/rome/tools/tree/lsp/v0.28.0/crates/rome_js_parser/src/lexer)
//!     * [rustc](https://github.com/rust-lang/rust/blob/1.82.0/compiler/rustc_lexer/src)
//!     * [v8](https://v8.dev/blog/scanner)

use std::{mem, num::NonZeroU32};

use rustc_hash::FxHashMap;

use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::RegExpFlags;
use oxc_span::{SourceType, Span};
use oxc_str::JSStr;

use crate::{
    UniquePromise,
    config::LexerConfig as Config,
    diagnostics::{self, ParserDiagnostic},
};

mod byte_handlers;
mod comment;
mod identifier;
mod jsx;
mod kind;
mod number;
mod numeric;
mod punctuation;
mod regex;
mod search;
mod source;
mod string;
mod template;
mod token;
mod trivia_builder;
mod typescript;
mod unicode;
mod whitespace;

#[cfg_attr(not(feature = "benchmarking"), expect(clippy::redundant_pub_crate))]
pub(crate) use byte_handlers::{ByteHandler, ByteHandlers, byte_handler_tables};
pub use kind::Kind;
pub use number::{parse_big_int, parse_float, parse_int};
pub use token::Token;

use source::{Source, SourcePosition};
use trivia_builder::TriviaBuilder;

#[derive(Debug)]
pub struct LexerCheckpoint<'a> {
    source_position: SourcePosition<'a>,
    errors_snapshot: ErrorSnapshot<'a>,
    tokens_len: usize,
    pure_comments: Option<(u32, NonZeroU32)>,
    no_side_effects_comments: Option<(u32, NonZeroU32)>,
}

#[derive(Debug)]
enum ErrorSnapshot<'a> {
    Empty,
    Count(usize),
    Full(Vec<ParserDiagnostic<'a>>),
}

/// Action to take when finishing a token.
/// Passed to [`Lexer::finish_next_inner`].
#[derive(Clone, Copy, Eq, PartialEq)]
enum FinishTokenMode {
    Push,
    Replace,
    Discard,
}

pub struct Lexer<'a, C: Config> {
    allocator: &'a Allocator,

    // Wrapper around source text. Must not be changed after initialization.
    source: Source<'a>,

    source_type: SourceType,

    /// The token currently being built.
    /// Only its start and flags are written here; its kind and end remain zero until
    /// `Token::finish` constructs the completed token.
    ///
    /// Between lexer calls, `token` is always `Token::default()`. Every method which produces a token
    /// ends in `finish_next_inner` or `finish_re_lex`, and both reset it.
    ///
    /// The one exception is before the first token is lexed, when it is `Token::new_on_new_line()`.
    /// `checkpoint` and `rewind` can never see that state, because the parser lexes the first token
    /// before it does anything else, and they are the only things which depend on `token` being
    /// `Token::default()`. Both assert it.
    token: Token,

    pub(crate) errors: Vec<ParserDiagnostic<'a>>,

    /// Errors that are only emitted if the file is determined to be a Module.
    /// For `ModuleKind::Unambiguous`, HTML-like comments are allowed during lexing,
    /// but if ESM syntax is found later, these comments become invalid.
    /// If resolved to Module → emit these errors.
    /// If resolved to Script → discard these errors.
    pub(crate) deferred_module_errors: Vec<ParserDiagnostic<'a>>,

    pub(crate) trivia_builder: TriviaBuilder<'a>,

    /// Decoded identifiers and string literals, indexed by [Token::start] when
    /// [Token::escaped] is true.
    pub escaped_strings: FxHashMap<u32, JSStr<'a>>,

    /// Data store for escaped templates, indexed by [Token::start] when [Token::escaped] is true
    /// `None` is saved when the string contains an invalid escape sequence.
    pub escaped_templates: FxHashMap<u32, Option<JSStr<'a>>>,

    /// Collected tokens in source order.
    tokens: ArenaVec<'a, Token>,

    /// Config
    pub(crate) config: C,
}

impl<'a, C: Config> Lexer<'a, C> {
    /// Create new `Lexer`.
    ///
    /// Requiring a `UniquePromise` to be provided guarantees only 1 `Lexer` can exist
    /// on a single thread at one time.
    pub(super) fn new(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        config: C,
        unique: UniquePromise,
    ) -> Self {
        let source = Source::new(source_text, unique);

        // If collecting tokens, allocate enough space so that the `Vec<Token>` will not have to grow during parsing.
        // `source_text.len() + 1` is almost always a large overestimate of number of tokens, but it's impossible to
        // have more than N + 1 tokens in a file which is N bytes long, so it'll never be an underestimate.
        //
        // + 1 is to account for the final `Eof` token. Without adding 1, the capacity could be too small for
        // minified files which have no space between any tokens. It would also be too small for empty files.
        //
        // Our largest benchmark file `binder.ts` is 190 KB, and `Token` is 16 bytes, so the `Vec<Token>`
        // would be ~3 MB even in the case of this unusually large file. That's not a huge amount of memory.
        //
        // However, we should choose a better heuristic based on real-world observation, and bring this usage down.
        let tokens = if config.tokens() {
            ArenaVec::with_capacity_in(source_text.len() + 1, &allocator)
        } else {
            ArenaVec::new_in(&allocator)
        };

        // The first token is at the start of file, so is allows on a new line
        let token = Token::new_on_new_line();
        Self {
            allocator,
            source,
            source_type,
            token,
            errors: vec![],
            deferred_module_errors: vec![],
            trivia_builder: TriviaBuilder::new_in(allocator),
            escaped_strings: FxHashMap::default(),
            escaped_templates: FxHashMap::default(),
            tokens,
            config,
        }
    }

    /// Backdoor to create a `Lexer` without holding a `UniquePromise`, for benchmarks.
    /// This function must NOT be exposed in public API as it breaks safety invariants.
    #[cfg(feature = "benchmarking")]
    pub fn new_for_benchmarks(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: SourceType,
        config: C,
    ) -> Self {
        let unique = UniquePromise::new_for_tests_and_benchmarks();
        Self::new(allocator, source_text, source_type, config, unique)
    }

    /// Get errors.
    /// Only used in benchmarks.
    #[cfg(feature = "benchmarking")]
    pub fn errors(&self) -> &[ParserDiagnostic<'a>] {
        &self.errors
    }

    /// Remaining string from `Source`
    pub fn remaining(&self) -> &'a str {
        self.source.remaining()
    }

    /// Creates a checkpoint storing the current lexer state.
    /// Use `rewind` to restore the lexer to the state stored in the checkpoint.
    pub fn checkpoint(&self) -> LexerCheckpoint<'a> {
        // `token` is always `Token::default()` here, so it does not need to be saved. See `token` field docs.
        debug_assert_eq!(self.token, Token::default());

        let errors_snapshot = if self.errors.is_empty() {
            ErrorSnapshot::Empty
        } else {
            ErrorSnapshot::Count(self.errors.len())
        };

        LexerCheckpoint {
            source_position: self.source.position(),
            errors_snapshot,
            tokens_len: self.tokens.len(),
            pure_comments: self.trivia_builder.previous_token_pure_comments(),
            no_side_effects_comments: self.trivia_builder.previous_token_no_side_effects_comments(),
        }
    }

    /// Create a checkpoint that can handle error popping.
    /// This is more expensive as it clones the errors vector.
    pub(crate) fn checkpoint_with_error_recovery(&self) -> LexerCheckpoint<'a> {
        // `token` is always `Token::default()` here, so it does not need to be saved. See `token` field docs.
        debug_assert_eq!(self.token, Token::default());

        let errors_snapshot = if self.errors.is_empty() {
            ErrorSnapshot::Empty
        } else {
            ErrorSnapshot::Full(self.errors.clone())
        };

        LexerCheckpoint {
            source_position: self.source.position(),
            errors_snapshot,
            tokens_len: self.tokens.len(),
            pure_comments: self.trivia_builder.previous_token_pure_comments(),
            no_side_effects_comments: self.trivia_builder.previous_token_no_side_effects_comments(),
        }
    }

    /// Rewinds the lexer to the same state as when the passed in `checkpoint` was created.
    pub fn rewind(&mut self, checkpoint: LexerCheckpoint<'a>) {
        // `token` is always `Token::default()` here, so it does not need to be restored. See `token` field docs.
        debug_assert_eq!(self.token, Token::default());

        match checkpoint.errors_snapshot {
            ErrorSnapshot::Empty => self.errors.clear(),
            ErrorSnapshot::Count(len) => self.errors.truncate(len),
            ErrorSnapshot::Full(errors) => self.errors = errors,
        }

        self.tokens.truncate(checkpoint.tokens_len);
        self.source.set_position(checkpoint.source_position);
        self.trivia_builder.set_pure_comments(checkpoint.pure_comments);
        self.trivia_builder.set_no_side_effects_comments(checkpoint.no_side_effects_comments);
    }

    pub fn peek_token(&mut self) -> Token {
        let checkpoint = self.checkpoint();
        let token = self.next_token();
        self.rewind(checkpoint);
        token
    }

    /// Returns `true` if the next token may start with an ASCII byte matching `matches_ascii`.
    /// Retains full lookahead for whitespace, comments, escapes, and non-ASCII bytes.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn next_token_may_start_with(&self, matches_ascii: impl FnOnce(u8) -> bool) -> bool {
        let mut pos = self.source.position();
        let end_pos = self.source.end();
        if pos >= end_pos {
            return false;
        }
        // SAFETY: `pos < end_pos`, and if `byte == b' '` we check `pos < end_pos` before reading again.
        let byte = unsafe {
            let b = pos.read();
            if b == b' ' {
                pos = pos.add(1);
                if pos >= end_pos {
                    return false;
                }
                pos.read()
            } else {
                b
            }
        };
        byte >= 0x80
            || matches!(byte, b' ' | b'\t'..=b'\r' | b'/' | b'<' | b'-' | b'\\')
            || matches_ascii(byte)
    }

    /// Read first token in file.
    pub fn first_token(&mut self) -> Token {
        // HashbangComment ::
        //     `#!` SingleLineCommentChars?
        if let Some([b'#', b'!']) = self.peek_2_bytes() {
            // SAFETY: Next 2 bytes are `#!`
            let kind = unsafe { self.read_hashbang_comment() };
            // Hashbangs are not included in tokens
            self.finish_next_inner(kind, FinishTokenMode::Discard)
        } else {
            let kind = self.read_next_token();
            self.finish_next(kind)
        }
    }

    /// Read next token in file.
    /// Use `first_token` for first token, and this method for all further tokens.
    #[inline]
    pub fn next_token(&mut self) -> Token {
        if let Some(token) = self.try_next_token() {
            return token;
        }
        self.next_token_slow()
    }

    #[inline(never)]
    pub(crate) fn next_token_slow(&mut self) -> Token {
        let kind = self.read_next_token();
        self.finish_next(kind)
    }

    /// Read the next token after `=` in a JSX attribute.
    pub(crate) fn next_jsx_attribute_value(&mut self) -> Token {
        let kind = self.read_next_jsx_attribute_value();
        self.finish_next(kind)
    }

    // This is a workaround for a problem where `next_token` is not inlined in lexer benchmark.
    // Must be kept in sync with `next_token` above, and contain exactly the same code.
    #[cfg(feature = "benchmarking")]
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn next_token_for_benchmarks(&mut self) -> Token {
        if let Some(token) = self.try_next_token() {
            return token;
        }
        self.next_token_slow()
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_fast_token(&mut self, token: Token) -> Token {
        debug_assert_eq!(self.token, Token::default());
        self.trivia_builder.clear_pure_comments();
        self.trivia_builder.clear_no_side_effects_comments();
        if self.config.tokens() {
            self.tokens.push_fast(token);
        }
        self.trivia_builder.handle_fast_token(token);
        token
    }

    /// Fast path for common tokens (ASCII identifiers/keywords, single-byte punctuation,
    /// dots, decimal integers, strings, and common operators) preceded by horizontal spaces/tabs
    /// or line breaks (`\n` / `\r\n`) with spaces/tabs/blank lines.
    ///
    /// On `None`, lexer state and `self.token` are unchanged so `read_next_token` runs normally.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn try_next_token(&mut self) -> Option<Token> {
        let mut pos = self.source.position();
        let end_pos = self.source.end();
        // SAFETY: `end_pos` is always >= `pos`.
        let remaining_bytes = unsafe { end_pos.offset_from(pos) };
        if remaining_bytes < 2 {
            return None;
        }

        // SAFETY: At least 2 bytes remain in `source`, and all pointer advances below check
        // `end_pos` bounds before reading.
        let (pos, byte, is_on_new_line) = unsafe {
            let mut byte = pos.read();
            let mut is_on_new_line = false;
            if byte == b' ' {
                pos = pos.add(1);
                byte = pos.read();
            } else if byte == b'\n' {
                is_on_new_line = true;
                pos = pos.add(1);
                if pos.read() == b' ' {
                    // Skip indentation spaces 8 bytes at a time using SWAR.
                    while end_pos.offset_from(pos) >= 8 {
                        let word = pos.slice(8);
                        let differing =
                            u64::from_le_bytes(word.try_into().unwrap()) ^ 0x2020_2020_2020_2020;
                        pos = pos.add((differing.trailing_zeros() >> 3) as usize);
                        if differing != 0 {
                            break;
                        }
                    }
                }
                while pos < end_pos && matches!(pos.read(), b' ' | b'\t' | b'\n') {
                    pos = pos.add(1);
                }
                if pos >= end_pos {
                    return None;
                }
                byte = pos.read();
            }
            (pos, byte, is_on_new_line)
        };

        match FAST_CLASS[byte as usize] {
            FastClass::Identifier => {
                self.scan_fast_ascii_identifier(pos, end_pos, byte, is_on_new_line)
            }
            FastClass::Punctuation => {
                let start = self.source.offset_of(pos);
                // SAFETY: `pos < end_pos` and `byte` is ASCII.
                self.source.set_position(unsafe { pos.add(1) });
                let kind = SIMPLE_PUNCT_KIND[byte as usize];
                Some(self.finish_fast_token(Token::new_fast(
                    start,
                    start + 1,
                    kind,
                    is_on_new_line,
                )))
            }
            FastClass::Dot => {
                // SAFETY: `pos < end_pos`, so `pos.add(1) <= end_pos`, and `next_pos < end_pos`
                // is checked before reading `next_pos`.
                let next_pos = unsafe {
                    let next_pos = pos.add(1);
                    if next_pos < end_pos {
                        let next = next_pos.read();
                        if next == b'.' || next.is_ascii_digit() {
                            return None;
                        }
                    }
                    next_pos
                };
                let start = self.source.offset_of(pos);
                self.source.set_position(next_pos);
                Some(self.finish_fast_token(Token::new_fast(
                    start,
                    start + 1,
                    Kind::Dot,
                    is_on_new_line,
                )))
            }
            FastClass::Digit => self.scan_fast_decimal_integer(pos, end_pos, byte, is_on_new_line),
            FastClass::Operator => self.scan_fast_operator(pos, end_pos, byte, is_on_new_line),
            FastClass::Slow => None,
        }
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn scan_fast_ascii_identifier(
        &mut self,
        start_pos: SourcePosition<'a>,
        end_pos: SourcePosition<'a>,
        first_byte: u8,
        is_on_new_line: bool,
    ) -> Option<Token> {
        // SAFETY: `start_pos < end_pos`, `first_byte` is ASCII, and all advances stay `<= end_pos`.
        let cur = unsafe {
            let mut cur = start_pos.add(1);
            // Fast scalar check for short identifiers (1-5 bytes) to avoid SIMD-to-GPR transfer latency.
            if end_pos.offset_from(cur) >= 4 {
                for _ in 0..4 {
                    let b = cur.read();
                    let class = ASCII_IDENTIFIER_CLASS[b as usize];
                    if class != IdentifierClass::Continue {
                        if class == IdentifierClass::Slow {
                            return None;
                        }
                        return Some(self.finish_fast_identifier(
                            start_pos,
                            cur,
                            first_byte,
                            is_on_new_line,
                        ));
                    }
                    cur = cur.add(1);
                }
            }
            while cur.can_read_batch_from(&self.source) {
                let slice = cur.slice(search::SEARCH_BATCH_SIZE);
                if let Some(i) = search::match_not_ascii_id_continue(slice.as_ptr(), false) {
                    cur = cur.add(i);
                    let b = cur.read();
                    if ASCII_IDENTIFIER_CLASS[b as usize] == IdentifierClass::Slow {
                        return None;
                    }
                    return Some(self.finish_fast_identifier(
                        start_pos,
                        cur,
                        first_byte,
                        is_on_new_line,
                    ));
                }
                cur = cur.add(search::SEARCH_BATCH_SIZE);
            }
            while cur < end_pos {
                let b = cur.read();
                let class = ASCII_IDENTIFIER_CLASS[b as usize];
                if class != IdentifierClass::Continue {
                    if class == IdentifierClass::Slow {
                        return None;
                    }
                    break;
                }
                cur = cur.add(1);
            }
            cur
        };
        Some(self.finish_fast_identifier(start_pos, cur, first_byte, is_on_new_line))
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_fast_identifier(
        &mut self,
        start_pos: SourcePosition<'a>,
        end_pos: SourcePosition<'a>,
        first_byte: u8,
        is_on_new_line: bool,
    ) -> Token {
        let start = self.source.offset_of(start_pos);
        let end = self.source.offset_of(end_pos);
        self.source.set_position(end_pos);
        let len = (end - start) as usize;
        // All keywords start with `'a'..='z'` (`>= b'a'` among identifier-start bytes)
        // and have lengths in `2..=11`.
        let kind = if first_byte >= b'a' && len.wrapping_sub(2) <= 9 {
            // SAFETY: `end_pos` is `len` bytes after `start_pos`.
            let lexeme = unsafe { start_pos.slice(len) };
            Kind::match_keyword_bytes(lexeme)
        } else {
            Kind::Ident
        };
        self.finish_fast_token(Token::new_fast(start, end, kind, is_on_new_line))
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn scan_fast_decimal_integer(
        &mut self,
        start_pos: SourcePosition<'a>,
        end_pos: SourcePosition<'a>,
        first_byte: u8,
        is_on_new_line: bool,
    ) -> Option<Token> {
        // SAFETY: `start_pos < end_pos`, `first_byte` is an ASCII digit, and `cur < end_pos` is
        // checked before every read/advance.
        let cur = unsafe {
            let mut cur = start_pos.add(1);
            if first_byte != b'0' {
                while cur < end_pos && cur.read().is_ascii_digit() {
                    cur = cur.add(1);
                }
            }
            if cur < end_pos {
                let next = cur.read();
                if ASCII_IDENTIFIER_CLASS[next as usize] != IdentifierClass::End || next == b'.' {
                    return None;
                }
            }
            cur
        };
        let start = self.source.offset_of(start_pos);
        let end = self.source.offset_of(cur);
        self.source.set_position(cur);
        Some(self.finish_fast_token(Token::new_fast(start, end, Kind::Decimal, is_on_new_line)))
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn scan_fast_operator(
        &mut self,
        start_pos: SourcePosition<'a>,
        end_pos: SourcePosition<'a>,
        c0: u8,
        is_on_new_line: bool,
    ) -> Option<Token> {
        // SAFETY: `end_pos.offset_from(start_pos) >= 1`, and `rem >= 2` / `rem >= 3` are checked
        // before reading `start_pos.add(1)` / `start_pos.add(2)`.
        let (len, kind) = unsafe {
            let rem = end_pos.offset_from(start_pos);
            let c1 = if rem >= 2 { start_pos.add(1).read() } else { 0 };
            match c0 {
                b'=' => match c1 {
                    b'=' => {
                        if rem >= 3 && start_pos.add(2).read() == b'=' {
                            (3, Kind::Eq3)
                        } else {
                            (2, Kind::Eq2)
                        }
                    }
                    b'>' => (2, Kind::Arrow),
                    _ => (1, Kind::Eq),
                },
                b'!' => match c1 {
                    b'=' => {
                        if rem >= 3 && start_pos.add(2).read() == b'=' {
                            (3, Kind::Neq2)
                        } else {
                            (2, Kind::Neq)
                        }
                    }
                    _ => (1, Kind::Bang),
                },
                b'+' => match c1 {
                    b'+' => (2, Kind::Plus2),
                    b'=' => (2, Kind::PlusEq),
                    _ => (1, Kind::Plus),
                },
                b'*' => match c1 {
                    b'*' => {
                        if rem >= 3 && start_pos.add(2).read() == b'=' {
                            (3, Kind::Star2Eq)
                        } else {
                            (2, Kind::Star2)
                        }
                    }
                    b'=' => (2, Kind::StarEq),
                    _ => (1, Kind::Star),
                },
                b'%' => {
                    if c1 == b'=' {
                        (2, Kind::PercentEq)
                    } else {
                        (1, Kind::Percent)
                    }
                }
                b'&' => match c1 {
                    b'&' => {
                        if rem >= 3 && start_pos.add(2).read() == b'=' {
                            (3, Kind::Amp2Eq)
                        } else {
                            (2, Kind::Amp2)
                        }
                    }
                    b'=' => (2, Kind::AmpEq),
                    _ => (1, Kind::Amp),
                },
                b'|' => match c1 {
                    b'|' => {
                        if rem >= 3 && start_pos.add(2).read() == b'=' {
                            (3, Kind::Pipe2Eq)
                        } else {
                            (2, Kind::Pipe2)
                        }
                    }
                    b'=' => (2, Kind::PipeEq),
                    _ => (1, Kind::Pipe),
                },
                b'^' => {
                    if c1 == b'=' {
                        (2, Kind::CaretEq)
                    } else {
                        (1, Kind::Caret)
                    }
                }
                b'<' => match c1 {
                    b'<' => {
                        if rem >= 3 && start_pos.add(2).read() == b'=' {
                            (3, Kind::ShiftLeftEq)
                        } else {
                            (2, Kind::ShiftLeft)
                        }
                    }
                    b'=' => (2, Kind::LtEq),
                    b'!' => return None,
                    _ => (1, Kind::LAngle),
                },
                b'?' => match c1 {
                    b'?' => {
                        if rem >= 3 && start_pos.add(2).read() == b'=' {
                            (3, Kind::Question2Eq)
                        } else {
                            (2, Kind::Question2)
                        }
                    }
                    b'.' => {
                        if rem >= 3
                            && start_pos.add(2).read() != b'.'
                            && start_pos.add(2).read().is_ascii_digit()
                        {
                            (1, Kind::Question)
                        } else {
                            (2, Kind::QuestionDot)
                        }
                    }
                    _ => (1, Kind::Question),
                },
                _ => unreachable!(),
            }
        };
        let start = self.source.offset_of(start_pos);
        // SAFETY: `len <= rem` and all matched bytes are ASCII.
        self.source.set_position(unsafe { start_pos.add(len as usize) });
        Some(self.finish_fast_token(Token::new_fast(start, start + len, kind, is_on_new_line)))
    }

    #[inline]
    fn finish_next(&mut self, kind: Kind) -> Token {
        self.finish_next_inner(kind, FinishTokenMode::Push)
    }

    #[inline]
    fn finish_next_retokenized(&mut self, kind: Kind) -> Token {
        self.finish_next_inner(kind, FinishTokenMode::Replace)
    }

    // `#[inline(always)]` to ensure is inlined into `finish_next` and `finish_next_retokenized`,
    // so that `mode` is statically known
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn finish_next_inner(&mut self, kind: Kind, mode: FinishTokenMode) -> Token {
        let token = self.token.finish(kind, self.offset());
        if self.config.tokens() {
            match mode {
                FinishTokenMode::Push => {
                    // We allocated sufficient capacity in the `Vec` for all tokens at the start.
                    // `push_fast` is optimized for the "doesn't need to grow" case.
                    self.tokens.push_fast(token);
                }
                FinishTokenMode::Replace => {
                    debug_assert!(
                        self.tokens.last().is_some_and(|last| last.start() == token.start())
                    );
                    let last = self.tokens.last_mut().unwrap();
                    *last = token;
                }
                FinishTokenMode::Discard => {}
            }
        }
        self.trivia_builder.handle_token(token);
        self.token = Token::default();
        token
    }

    /// Finish a re-lexed token used only for parser disambiguation.
    /// This must not mutate the externally collected token stream.
    fn finish_re_lex(&mut self, kind: Kind) -> Token {
        let token = self.token.finish(kind, self.offset());
        self.token = Token::default();
        token
    }

    /// Overwrite the last token in the collected token stream.
    ///
    /// Used by `re_lex_as_typescript_l_angle` to replace a compound token (`<<`, `<=` or `<<=`)
    /// with the single `<` it re-lexed, and to restore the compound token when `try_parse`
    /// fails and rewinds.
    #[inline]
    pub(crate) fn rewrite_last_collected_token(&mut self, token: Token) {
        // Make this function a no-op when tokens are statically disabled (`NoTokensLexerConfig`)
        if C::TOKENS_METHOD_IS_STATIC && !self.config.tokens() {
            return;
        }

        // Because of the static check above, there's no need to check `self.config.tokens()` here.
        //
        // * If tokens are statically disabled, we already exited.
        // * If tokens are statically enabled, then `self.tokens` is always non-empty.
        // * If tokens are runtime disabled, then `self.tokens` is always empty.
        // * If tokens are runtime enabled, then `self.tokens` is always non-empty.
        //
        // So checking `self.config.tokens()` too here would be redundant,
        // and would be an extra branch with runtime config (`RuntimeLexerConfig`).
        if let Some(last) = self.tokens.last_mut() {
            *last = token;
        } else {
            // When tokens are enabled, this should be unreachable
            debug_assert!(!self.config.tokens());
        }
    }

    pub(crate) fn take_tokens(&mut self) -> ArenaVec<'a, Token> {
        mem::replace(&mut self.tokens, ArenaVec::new_in(&self.allocator))
    }

    pub(crate) fn set_tokens(&mut self, tokens: ArenaVec<'a, Token>) {
        self.tokens = tokens;
    }

    /// Finalize tokens and return them.
    /// Called at very end of parsing.
    pub(crate) fn finalize_tokens(&mut self) -> ArenaVec<'a, Token> {
        if self.config.tokens() {
            // Tokens are enabled. Discard the last token, which marks the end of input.
            // Usually that is `Eof`. Where the lexer cannot lex any further - an unterminated string,
            // for example - it emits `Undetermined` instead, and no `Eof` ever follows.
            // The parser treats the two alike, and neither is a real token, so both are discarded.
            let mut tokens = self.take_tokens();
            let last_token = tokens.pop();
            debug_assert!(
                last_token
                    .is_some_and(|token| matches!(token.kind(), Kind::Eof | Kind::Undetermined))
            );
            tokens
        } else {
            // Tokens are disabled. Just return an empty vec.
            debug_assert!(self.tokens.is_empty());
            ArenaVec::new_in(&self.allocator)
        }
    }

    /// Advance source cursor to end of file.
    #[inline]
    pub fn advance_to_end(&mut self) {
        self.source.advance_to_end();
    }

    // ---------- Private Methods ---------- //
    fn error(&mut self, error: ParserDiagnostic<'a>) {
        self.errors.push(error);
    }

    /// Get the length offset from the source, in UTF-8 bytes
    #[inline]
    fn offset(&self) -> u32 {
        self.source.offset()
    }

    /// Get the current unterminated token range
    fn unterminated_range(&self) -> Span {
        Span::new(self.token.start(), self.offset())
    }

    /// Consume the current char if not at EOF
    #[inline]
    fn next_char(&mut self) -> Option<char> {
        self.source.next_char()
    }

    /// Consume the current char
    #[inline]
    fn consume_char(&mut self) -> char {
        self.source.next_char().unwrap()
    }

    /// Consume the current char and the next if not at EOF
    #[inline]
    fn next_2_chars(&mut self) -> Option<[char; 2]> {
        self.source.next_2_chars()
    }

    /// Consume the current char and the next
    #[inline]
    fn consume_2_chars(&mut self) -> [char; 2] {
        self.next_2_chars().unwrap()
    }

    /// Peek the next byte without advancing the position
    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.source.peek_byte()
    }

    /// Peek the next two bytes without advancing the position
    #[inline]
    fn peek_2_bytes(&self) -> Option<[u8; 2]> {
        self.source.peek_2_bytes()
    }

    /// Peek the next char without advancing the position
    #[inline]
    fn peek_char(&self) -> Option<char> {
        self.source.peek_char()
    }

    /// Peek the next byte, and advance the current position if it matches
    /// the given ASCII char.
    // `#[inline(always)]` to make sure the `assert!` gets optimized out.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn next_ascii_byte_eq(&mut self, b: u8) -> bool {
        // TODO: can be replaced by `std::ascii:Char` once stabilized.
        // https://github.com/rust-lang/rust/issues/110998
        assert!(b.is_ascii());
        // SAFETY: `b` is a valid ASCII char.
        unsafe { self.source.advance_if_ascii_eq(b) }
    }

    fn current_offset(&self) -> Span {
        Span::empty(self.offset())
    }

    /// Return `IllegalCharacter` Error or `UnexpectedEnd` if EOF
    fn unexpected_err(&mut self) {
        let offset = self.current_offset();
        match self.peek_char() {
            Some(c) => self.error(diagnostics::invalid_character(c, offset)),
            None => self.error(diagnostics::unexpected_end(offset)),
        }
    }

    /// Read each char and set the current token
    /// Whitespace and line terminators are skipped
    #[inline] // Make sure is inlined into `next_token`
    fn read_next_token(&mut self) -> Kind {
        self.trivia_builder.clear_pure_comments();
        self.trivia_builder.clear_no_side_effects_comments();

        let end_pos = self.source.end();
        loop {
            // Single spaces between tokens are common, so consume a space before processing the next token.
            // Can only do this if there are at least 2 bytes left in source.
            // If there aren't 2 bytes left, delegate to `read_next_token_at_end` (cold branch).
            let mut pos = self.source.position();
            // SAFETY: `source.end()` is always equal to or after `source.position()`
            let remaining_bytes = unsafe { end_pos.offset_from(pos) };
            if remaining_bytes >= 2 {
                // SAFETY: There are at least 2 bytes remaining in source.
                let mut byte = unsafe { pos.read() };

                // Only update the cursor and read again when consuming a space.
                if byte == b' ' {
                    // SAFETY: At least 2 bytes remain, so advancing past one ASCII space
                    // leaves at least one byte to read.
                    pos = unsafe { pos.add(1) };
                    self.source.set_position(pos);
                    // SAFETY: At least one byte remains after the space.
                    byte = unsafe { pos.read() };
                }

                // Set token start
                let offset = self.source.offset_of(pos);
                self.token.set_start(offset);

                // SAFETY: `byte` is byte value at current position in source
                let kind = unsafe { self.handle_byte(byte) };
                if kind != Kind::Skip {
                    return kind;
                }
            } else {
                // Only 0 or 1 bytes left in source.
                // Delegate to `#[cold]` function as this is a very rare case.
                return self.read_next_token_at_end();
            }
        }
    }

    /// Cold path for reading next token where only 0 or 1 bytes are left in source.
    #[inline(never)]
    #[cold]
    fn read_next_token_at_end(&mut self) -> Kind {
        let offset = self.offset();
        self.token.set_start(offset);

        if let Some(byte) = self.peek_byte() {
            // SAFETY: `byte` is byte value at current position in source
            let kind = unsafe { self.handle_byte(byte) };
            if kind != Kind::Skip {
                return kind;
            }
            // Last byte was whitespace/line break (`Kind::Skip`), so now at EOF
            self.token.set_start(offset + 1);
        }

        Kind::Eof
    }

    #[inline]
    fn read_next_jsx_attribute_value(&mut self) -> Kind {
        self.trivia_builder.clear_pure_comments();
        self.trivia_builder.clear_no_side_effects_comments();

        let end_pos = self.source.end();
        loop {
            let mut pos = self.source.position();
            // SAFETY: `source.end()` is always equal to or after `source.position()`
            let remaining_bytes = unsafe { end_pos.offset_from(pos) };
            if remaining_bytes >= 2 {
                // SAFETY: There are at least 2 bytes remaining in source.
                let mut byte = unsafe { pos.read() };

                // Only update the cursor and read again when consuming a space.
                if byte == b' ' {
                    // SAFETY: At least 2 bytes remain, so advancing past one ASCII space
                    // leaves at least one byte to read.
                    pos = unsafe { pos.add(1) };
                    self.source.set_position(pos);
                    // SAFETY: At least one byte remains after the space.
                    byte = unsafe { pos.read() };
                }

                let offset = self.source.offset_of(pos);
                self.token.set_start(offset);

                if matches!(byte, b'"' | b'\'') {
                    // SAFETY: `byte` is the current ASCII quote delimiter.
                    return unsafe { self.read_jsx_string_literal(byte) };
                }

                // SAFETY: `byte` is byte value at current position in source.
                let kind = unsafe { self.handle_byte(byte) };
                if kind != Kind::Skip {
                    return kind;
                }
            } else {
                return self.read_next_jsx_attribute_value_at_end();
            }
        }
    }

    #[inline(never)]
    #[cold]
    fn read_next_jsx_attribute_value_at_end(&mut self) -> Kind {
        let offset = self.offset();
        self.token.set_start(offset);

        if let Some(byte) = self.peek_byte() {
            if matches!(byte, b'"' | b'\'') {
                // SAFETY: `byte` is the current ASCII quote delimiter.
                return unsafe { self.read_jsx_string_literal(byte) };
            }

            // SAFETY: `byte` is byte value at current position in source.
            let kind = unsafe { self.handle_byte(byte) };
            if kind != Kind::Skip {
                return kind;
            }
            self.token.set_start(offset + 1);
        }

        Kind::Eof
    }
}

/// Call a closure while hinting to compiler that this branch is rarely taken.
///
/// Unlike [`cold_path`], which only guides branch layout, this `#[cold]` trampoline also moves
/// the closure's code and stack usage out of the caller's frame (unless compiler chooses to
/// inline it). Use `cold_path` for pure branch hints. Use `cold_branch` where keeping large
/// stack objects (e.g. an `OxcDiagnostic` return buffer) out of the hot function's stack frame
/// is the aim.
///
/// "Cold trampoline function", suggested in:
/// <https://users.rust-lang.org/t/is-cold-the-only-reliable-way-to-hint-to-branch-predictor/106509/2>
///
/// [`cold_path`]: oxc_data_structures::branch_hints::cold_path
#[cold]
pub fn cold_branch<F: FnOnce() -> T, T>(f: F) -> T {
    f()
}

const SIMPLE_PUNCT_KIND: [Kind; 256] = {
    let mut table = [Kind::Eof; 256];
    table[b'~' as usize] = Kind::Tilde;
    table[b'(' as usize] = Kind::LParen;
    table[b')' as usize] = Kind::RParen;
    table[b'{' as usize] = Kind::LCurly;
    table[b'}' as usize] = Kind::RCurly;
    table[b'[' as usize] = Kind::LBrack;
    table[b']' as usize] = Kind::RBrack;
    table[b';' as usize] = Kind::Semicolon;
    table[b',' as usize] = Kind::Comma;
    table[b':' as usize] = Kind::Colon;
    table[b'@' as usize] = Kind::At;
    table[b'>' as usize] = Kind::RAngle;
    table
};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum FastClass {
    Slow,
    Identifier,
    Punctuation,
    Digit,
    Dot,
    Operator,
}

const FAST_CLASS: [FastClass; 256] = {
    let mut table = [FastClass::Slow; 256];
    let mut byte = 0;
    while byte < 256 {
        if (byte >= b'a' as usize && byte <= b'z' as usize)
            || (byte >= b'A' as usize && byte <= b'Z' as usize)
            || byte == b'_' as usize
            || byte == b'$' as usize
        {
            table[byte] = FastClass::Identifier;
        } else if !matches!(SIMPLE_PUNCT_KIND[byte], Kind::Eof) {
            table[byte] = FastClass::Punctuation;
        }
        byte += 1;
    }
    let mut digit = b'0' as usize;
    while digit <= b'9' as usize {
        table[digit] = FastClass::Digit;
        digit += 1;
    }
    table[b'.' as usize] = FastClass::Dot;
    let operators = b"+*%<=!&|^?";
    let mut i = 0;
    while i < operators.len() {
        table[operators[i] as usize] = FastClass::Operator;
        i += 1;
    }
    table
};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum IdentifierClass {
    Continue,
    End,
    Slow,
}

const ASCII_IDENTIFIER_CLASS: [IdentifierClass; 256] = {
    let mut table = [IdentifierClass::End; 256];
    let mut byte = 0;
    while byte < 256 {
        if (byte >= b'0' as usize && byte <= b'9' as usize)
            || (byte >= b'A' as usize && byte <= b'Z' as usize)
            || (byte >= b'a' as usize && byte <= b'z' as usize)
            || byte == b'_' as usize
            || byte == b'$' as usize
        {
            table[byte] = IdentifierClass::Continue;
        } else if byte == b'\\' as usize || byte >= 0x80 {
            table[byte] = IdentifierClass::Slow;
        }
        byte += 1;
    }
    table
};

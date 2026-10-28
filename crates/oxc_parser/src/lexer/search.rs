//! Structs and macros for searching source for combinations of byte values.
//!
//! * `SafeByteMatchTable` is a lookup table type for byte values.
//! * The `safe_byte_match_table!` macro creates those tables at compile time.
//! * The `byte_search!` macro searches source text for the first byte matching a
//!   `SafeByteMatchTable`.

/// Batch size for searching (16 bytes = 1 SIMD vector width on NEON/SSE2/WASM SIMD128)
pub const SEARCH_BATCH_SIZE: usize = 16;

/// Compile-time classification of a [`SafeByteMatchTable`] for SIMD acceleration.
#[derive(Clone, Copy)]
pub enum SimdSearchKind {
    /// Match any byte equal to one of `b0`, `b1`, `b2`, `b3`.
    Any4(u8, u8, u8, u8),
    /// Match any byte NOT equal to any of `b0`, `b1`, `b2`, `b3`.
    Not4(u8, u8, u8, u8),
    /// Match any byte that is NOT an ASCII identifier continuation (`a-z`, `A-Z`, `0-9`, `_`, `$`).
    NotAsciiIdContinue,
    /// Match any byte that is NOT an ASCII JSX identifier continuation (`a-z`, `A-Z`, `0-9`, `_`, `$`, `-`).
    NotAsciiJsxIdContinue,
    /// Fallback to scalar table lookup.
    Table,
}

/// Safe byte matcher lookup table.
///
/// Create table at compile time as a `static` or `const` with `safe_byte_match_table!` macro.
/// Test bytes against table with `SafeByteMatchTable::matches`.
/// Or use `byte_search!` macro to search for first matching byte in source.
///
/// `byte_search!` using this table is guaranteed to leave `lexer.source` positioned on a UTF-8
/// character boundary, provided that its starting position is already on a UTF-8 character
/// boundary.
///
/// To make this guarantee, one of the following must be true:
///
/// 1. Table contains `true` for all byte values 192 - 247
///    i.e. first byte of any multi-byte Unicode character matches.
///    (NB: 248 - 255 cannot occur in UTF-8 strings)
///    e.g.
///    * `safe_byte_match_table!(|b| b >= 192)`
///    * `safe_byte_match_table!(|b| !b.is_ascii())`
///
/// 2. Table contains `false` for all byte values 128 - 191
///    i.e. the continuation bytes of any multi-byte Unicode chars will be consumed in full.
///    e.g.
///    * `safe_byte_match_table!(|b| b < 128 || b >= 192)`
///    * `safe_byte_match_table!(|b| b.is_ascii())`
///    * `safe_byte_match_table!(|b| b == ' ' || b == '\t')`
///
/// This is statically checked by `SafeByteMatchTable::new`, and will fail to compile if match
/// pattern does not satisfy one of the above.
///
/// # Examples
/// ```rust,ignore
/// use crate::lexer::search::{SafeByteMatchTable, safe_byte_match_table};
///
/// static NOT_ASCII: SafeByteMatchTable = safe_byte_match_table!(|b| !b.is_ascii());
/// assert_eq!(NOT_ASCII.matches(b'X'), false);
/// assert_eq!(NOT_ASCII.matches(192), true);
///
/// impl<'a> Lexer<'a> {
///   fn eat_ascii(&mut self) {
///     // NB: Using `byte_search!` macro with a `SafeByteMatchTable` is safe
///     byte_search! {
///       lexer: self,
///       table: NOT_ASCII,
///       handle_match: |matched_byte, start| {},
///       handle_eof: |start| {},
///     };
///   }
/// }
/// ```
#[repr(C, align(64))]
pub struct SafeByteMatchTable {
    bytes: [bool; 256],
    simd_kind: SimdSearchKind,
}

impl SafeByteMatchTable {
    // Create new `SafeByteMatchTable`.
    pub const fn new(bytes: [bool; 256]) -> Self {
        // Check if contains either:
        // 1. `true` for all byte values 192..248
        // 2. `false` for all byte values 128..192
        let mut unicode_start_all_match = true;
        let mut unicode_cont_all_no_match = true;

        let mut true_count = 0usize;
        let mut false_count = 0usize;
        let mut true_bytes = [0u8; 4];
        let mut false_bytes = [0u8; 4];
        let mut is_not_ascii_id_continue = true;
        let mut is_not_ascii_jsx_id_continue = true;

        let mut i = 0;
        loop {
            let matches = bytes[i];
            #[expect(clippy::cast_possible_truncation)]
            let b = i as u8;

            if matches {
                if true_count < 4 {
                    true_bytes[true_count] = b;
                }
                true_count += 1;
                if i >= 128 && i < 192 {
                    unicode_cont_all_no_match = false;
                }
            } else {
                if false_count < 4 {
                    false_bytes[false_count] = b;
                }
                false_count += 1;
                if i >= 192 && i < 248 {
                    unicode_start_all_match = false;
                }
            }

            let is_id_cont = b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
            if matches == is_id_cont {
                is_not_ascii_id_continue = false;
            }
            if matches == (is_id_cont || b == b'-') {
                is_not_ascii_jsx_id_continue = false;
            }

            i += 1;
            if i == 256 {
                break;
            }
        }

        assert!(
            unicode_start_all_match || unicode_cont_all_no_match,
            "Cannot create a `SafeByteMatchTable` with an unsafe pattern"
        );

        let simd_kind = if true_count >= 1 && true_count <= 4 {
            let b0 = true_bytes[0];
            let b1 = if true_count > 1 { true_bytes[1] } else { b0 };
            let b2 = if true_count > 2 { true_bytes[2] } else { b0 };
            let b3 = if true_count > 3 { true_bytes[3] } else { b0 };
            SimdSearchKind::Any4(b0, b1, b2, b3)
        } else if false_count >= 1 && false_count <= 4 {
            let b0 = false_bytes[0];
            let b1 = if false_count > 1 { false_bytes[1] } else { b0 };
            let b2 = if false_count > 2 { false_bytes[2] } else { b0 };
            let b3 = if false_count > 3 { false_bytes[3] } else { b0 };
            SimdSearchKind::Not4(b0, b1, b2, b3)
        } else if is_not_ascii_id_continue {
            SimdSearchKind::NotAsciiIdContinue
        } else if is_not_ascii_jsx_id_continue {
            SimdSearchKind::NotAsciiJsxIdContinue
        } else {
            SimdSearchKind::Table
        };

        Self { bytes, simd_kind }
    }

    /// Declare that using this table for searching is safe.
    /// `byte_search!` calls `.use_table()` on the table it is provided.
    #[expect(clippy::unused_self)]
    #[inline]
    pub const fn use_table(&self) {}

    /// Test a value against this `SafeByteMatchTable`.
    #[inline]
    pub const fn matches(&self, b: u8) -> bool {
        self.bytes[b as usize]
    }

    /// Search a 16-byte batch starting at `batch` for the first matching byte offset `0..16`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn search_batch(&self, batch: &[u8]) -> Option<usize> {
        debug_assert!(batch.len() >= SEARCH_BATCH_SIZE);
        match self.simd_kind {
            SimdSearchKind::NotAsciiIdContinue => {
                // SAFETY: `batch` has at least 16 bytes.
                unsafe { simd::match_not_ascii_id_continue(batch.as_ptr(), false) }
            }
            SimdSearchKind::NotAsciiJsxIdContinue => {
                // SAFETY: `batch` has at least 16 bytes.
                unsafe { simd::match_not_ascii_id_continue(batch.as_ptr(), true) }
            }
            SimdSearchKind::Any4(b0, b1, b2, b3) => {
                // SAFETY: `batch` has at least 16 bytes.
                unsafe { simd::match_any4(batch.as_ptr(), b0, b1, b2, b3) }
            }
            SimdSearchKind::Not4(b0, b1, b2, b3) => {
                // SAFETY: `batch` has at least 16 bytes.
                unsafe { simd::match_not4(batch.as_ptr(), b0, b1, b2, b3) }
            }
            SimdSearchKind::Table => {
                for (i, &byte) in batch[..SEARCH_BATCH_SIZE].iter().enumerate() {
                    if self.matches(byte) {
                        return Some(i);
                    }
                }
                None
            }
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod simd {
    use std::arch::aarch64::{
        uint8x16_t, vandq_u8, vceqq_u8, vcltq_u8, vdupq_n_u8, vget_lane_u64, vld1q_u8, vmvnq_u8,
        vorrq_u8, vreinterpret_u64_u8, vreinterpretq_u16_u8, vshrn_n_u16, vsubq_u8,
    };

    #[expect(clippy::inline_always)]
    #[inline(always)]
    unsafe fn load16(ptr: *const u8) -> uint8x16_t {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        unsafe { vld1q_u8(ptr) }
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    unsafe fn first_true_lane(mask: uint8x16_t) -> Option<usize> {
        // Pack 4 bits per lane with SHRN: aarch64 has no byte movemask.
        // SAFETY: NEON is mandatory on aarch64.
        let bits = unsafe {
            let nibbles = vshrn_n_u16::<4>(vreinterpretq_u16_u8(mask));
            vget_lane_u64::<0>(vreinterpret_u64_u8(nibbles))
        };
        if bits == 0 {
            None
        } else {
            let lane = if cfg!(target_endian = "little") {
                (bits.trailing_zeros() >> 2) as usize
            } else {
                (bits.leading_zeros() >> 2) as usize
            };
            Some(lane)
        }
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_any4(ptr: *const u8, b0: u8, b1: u8, b2: u8, b3: u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        unsafe {
            let chunk = load16(ptr);
            let m0 = vceqq_u8(chunk, vdupq_n_u8(b0));
            let m1 = vceqq_u8(chunk, vdupq_n_u8(b1));
            let m2 = vceqq_u8(chunk, vdupq_n_u8(b2));
            let m3 = vceqq_u8(chunk, vdupq_n_u8(b3));
            let mask = vorrq_u8(vorrq_u8(m0, m1), vorrq_u8(m2, m3));
            first_true_lane(mask)
        }
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_not4(ptr: *const u8, b0: u8, b1: u8, b2: u8, b3: u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        unsafe {
            let chunk = load16(ptr);
            let m0 = vceqq_u8(chunk, vdupq_n_u8(b0));
            let m1 = vceqq_u8(chunk, vdupq_n_u8(b1));
            let m2 = vceqq_u8(chunk, vdupq_n_u8(b2));
            let m3 = vceqq_u8(chunk, vdupq_n_u8(b3));
            let eq = vorrq_u8(vorrq_u8(m0, m1), vorrq_u8(m2, m3));
            first_true_lane(vmvnq_u8(eq))
        }
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_not_ascii_id_continue(ptr: *const u8, allow_hyphen: bool) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        unsafe {
            let chunk = load16(ptr);
            let lower = vorrq_u8(chunk, vdupq_n_u8(0x20));
            let alpha = vcltq_u8(vsubq_u8(lower, vdupq_n_u8(b'a')), vdupq_n_u8(26));
            let digit = vcltq_u8(vsubq_u8(chunk, vdupq_n_u8(b'0')), vdupq_n_u8(10));
            let under = vceqq_u8(chunk, vdupq_n_u8(b'_'));
            let dollar = vceqq_u8(chunk, vdupq_n_u8(b'$'));
            let mut valid = vorrq_u8(vorrq_u8(alpha, digit), vorrq_u8(under, dollar));
            if allow_hyphen {
                valid = vorrq_u8(valid, vceqq_u8(chunk, vdupq_n_u8(b'-')));
            }
            first_true_lane(vmvnq_u8(valid))
        }
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_star_slash(ptr: *const u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 17 bytes are readable at `ptr`.
        unsafe {
            let stars = vceqq_u8(load16(ptr), vdupq_n_u8(b'*'));
            let slashes = vceqq_u8(load16(ptr.add(1)), vdupq_n_u8(b'/'));
            first_true_lane(vandq_u8(stars, slashes))
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod simd {
    use std::arch::x86_64::{
        __m128i, _mm_and_si128, _mm_cmpeq_epi8, _mm_cmpgt_epi8, _mm_loadu_si128, _mm_movemask_epi8,
        _mm_or_si128, _mm_set1_epi8, _mm_sub_epi8,
    };

    #[expect(clippy::inline_always, clippy::cast_ptr_alignment)]
    #[inline(always)]
    unsafe fn load16(ptr: *const u8) -> __m128i {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        unsafe { _mm_loadu_si128(ptr.cast::<__m128i>()) }
    }

    #[expect(clippy::inline_always, clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[inline(always)]
    unsafe fn first_true_lane(mask: __m128i) -> Option<usize> {
        // SAFETY: SSE2 is mandatory on x86_64.
        let bits = unsafe { _mm_movemask_epi8(mask) } as u16;
        if bits == 0 { None } else { Some(bits.trailing_zeros() as usize) }
    }

    #[expect(clippy::inline_always, clippy::cast_possible_wrap)]
    #[inline(always)]
    pub unsafe fn match_any4(ptr: *const u8, b0: u8, b1: u8, b2: u8, b3: u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`, and SSE2 is mandatory on x86_64.
        unsafe {
            let chunk = load16(ptr);
            let m0 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b0 as i8));
            let m1 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b1 as i8));
            let m2 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b2 as i8));
            let m3 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b3 as i8));
            let mask = _mm_or_si128(_mm_or_si128(m0, m1), _mm_or_si128(m2, m3));
            first_true_lane(mask)
        }
    }

    #[expect(
        clippy::inline_always,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    #[inline(always)]
    pub unsafe fn match_not4(ptr: *const u8, b0: u8, b1: u8, b2: u8, b3: u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`, and SSE2 is mandatory on x86_64.
        unsafe {
            let chunk = load16(ptr);
            let m0 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b0 as i8));
            let m1 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b1 as i8));
            let m2 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b2 as i8));
            let m3 = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b3 as i8));
            let eq = _mm_or_si128(_mm_or_si128(m0, m1), _mm_or_si128(m2, m3));
            let bits = (_mm_movemask_epi8(eq) as u16) ^ 0xFFFF;
            if bits == 0 { None } else { Some(bits.trailing_zeros() as usize) }
        }
    }

    #[expect(
        clippy::inline_always,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    #[inline(always)]
    pub unsafe fn match_not_ascii_id_continue(ptr: *const u8, allow_hyphen: bool) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`, and SSE2 is mandatory on x86_64.
        unsafe {
            let chunk = load16(ptr);
            let lower = _mm_or_si128(chunk, _mm_set1_epi8(0x20));
            let alpha = _mm_cmpgt_epi8(
                _mm_set1_epi8(-102),
                _mm_sub_epi8(lower, _mm_set1_epi8((b'a' as i8).wrapping_sub(i8::MIN))),
            );
            let digit = _mm_cmpgt_epi8(
                _mm_set1_epi8(-118),
                _mm_sub_epi8(chunk, _mm_set1_epi8((b'0' as i8).wrapping_sub(i8::MIN))),
            );
            let under = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'_' as i8));
            let dollar = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'$' as i8));
            let mut valid = _mm_or_si128(_mm_or_si128(alpha, digit), _mm_or_si128(under, dollar));
            if allow_hyphen {
                valid = _mm_or_si128(valid, _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'-' as i8)));
            }
            let bits = (_mm_movemask_epi8(valid) as u16) ^ 0xFFFF;
            if bits == 0 { None } else { Some(bits.trailing_zeros() as usize) }
        }
    }

    #[expect(clippy::inline_always, clippy::cast_possible_wrap)]
    #[inline(always)]
    pub unsafe fn match_star_slash(ptr: *const u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 17 bytes are readable at `ptr`, and SSE2 is mandatory on x86_64.
        unsafe {
            let stars = _mm_cmpeq_epi8(load16(ptr), _mm_set1_epi8(b'*' as i8));
            let slashes = _mm_cmpeq_epi8(load16(ptr.add(1)), _mm_set1_epi8(b'/' as i8));
            first_true_lane(_mm_and_si128(stars, slashes))
        }
    }
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
mod simd {
    use super::SEARCH_BATCH_SIZE;

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_any4(ptr: *const u8, b0: u8, b1: u8, b2: u8, b3: u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        let slice = unsafe { std::slice::from_raw_parts(ptr, SEARCH_BATCH_SIZE) };
        slice.iter().position(|&b| b == b0 || b == b1 || b == b2 || b == b3)
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_not4(ptr: *const u8, b0: u8, b1: u8, b2: u8, b3: u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        let slice = unsafe { std::slice::from_raw_parts(ptr, SEARCH_BATCH_SIZE) };
        slice.iter().position(|&b| b != b0 && b != b1 && b != b2 && b != b3)
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_not_ascii_id_continue(ptr: *const u8, allow_hyphen: bool) -> Option<usize> {
        // SAFETY: Caller guarantees at least 16 bytes are readable at `ptr`.
        let slice = unsafe { std::slice::from_raw_parts(ptr, SEARCH_BATCH_SIZE) };
        slice.iter().position(|&b| {
            !(b.is_ascii_alphanumeric() || b == b'_' || b == b'$' || (allow_hyphen && b == b'-'))
        })
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn match_star_slash(ptr: *const u8) -> Option<usize> {
        // SAFETY: Caller guarantees at least 17 bytes are readable at `ptr`.
        let slice = unsafe { std::slice::from_raw_parts(ptr, SEARCH_BATCH_SIZE + 1) };
        (0..SEARCH_BATCH_SIZE).find(|&i| slice[i] == b'*' && slice[i + 1] == b'/')
    }
}

pub(super) use simd::{match_not_ascii_id_continue, match_star_slash};

/// Macro to create a `SafeByteMatchTable` at compile time.
///
/// `safe_byte_match_table!(|b| !b.is_ascii())` expands to:
///
/// ```rust,ignore
/// {
///   use crate::lexer::search::SafeByteMatchTable;
///   #[allow(clippy::eq_op, clippy::allow_attributes)]
///   const TABLE: SafeByteMatchTable = SafeByteMatchTable::new([
///     (!0u8.is_ascii()),
///     (!1u8.is_ascii()),
///     /* ... */
///     (!255u8.is_ascii()),
///   ]);
///   TABLE
/// }
/// ```
macro_rules! safe_byte_match_table {
    (|$byte:ident| $res:expr) => {{
        use crate::lexer::search::SafeByteMatchTable;
        // Clippy creates warnings because e.g. `safe_byte_match_table!(|b| b == 0)`
        // is expanded to `SafeByteMatchTable([0 == 0, ... ])`
        #[allow(clippy::eq_op, clippy::allow_attributes)]
        const TABLE: SafeByteMatchTable = seq_macro::seq!($byte in 0u8..=255 {
            SafeByteMatchTable::new([#($res,)*])
        });
        TABLE
    }};
}
pub(crate) use safe_byte_match_table;

/// Macro to search for first byte matching a `SafeByteMatchTable`.
///
/// Search processes source in batches of `SEARCH_BATCH_SIZE` bytes for speed.
/// When not enough bytes remaining in source for a batch, search source byte by byte.
///
/// This is a macro rather than a function because searching is a bit faster when all the code
/// is in a single function, and some parts (e.g. `continue_if`) can be statically removed by
/// the compiler if they're not used.
///
/// Used as follows:
///
/// ```rust,ignore
/// static NOT_STUFF_TABLE: SafeByteMatchTable = safe_byte_match_table!(|b| !is_stuff(b));
///
/// impl<'a> Lexer<'a> {
///   fn eat_stuff(&mut self) -> bool {
///     let matched_byte = byte_search! {
///       lexer: self,
///       table: NOT_STUFF_TABLE,
///       handle_eof: {
///         // No bytes from start position to end of source matched the table.
///         // `lexer.source` is now positioned at EOF.
///         // Evaluate to a `u8` which macro call will evaluate to.
///         0xFF
///         // Or can `return` from enclosing function e.g. `return false;`
///       },
///     };
///
///     // Matching byte has been found.
///     // `matched_byte` is `u8` value of first byte which matched the table
///     // (or `0xFF` if EOF, because `handle_eof` evaluates to `0xFF`).
///     // `lexer.source` is now positioned on first matching byte.
///     // Handle the next matching byte (deal with any special cases).
///     matched_byte == b'X'
///   }
/// }
/// ```
///
/// or provide the `SourcePosition` to start searching from:
///
/// ```rust,ignore
/// impl<'a> Lexer<'a> {
///   fn eat_stuff(&mut self) -> bool {
///     let start = unsafe { self.source.position().add(1) };
///     let matched_byte = byte_search! {
///       lexer: self,
///       table: NOT_STUFF_TABLE,
///       start: start,
///       handle_eof: {
///         // No bytes from start position to end of source matched the table.
///         // `lexer.source` is now positioned at EOF.
///         return false;
///       },
///     };
///
///     // Matching byte has been found.
///     // `matched_byte` is `u8` value of first byte which matched the table.
///     // `lexer.source` is now positioned on first matching byte.
///     // Handle the next matching byte (deal with any special cases).
///     matched_byte == b'X'
///   }
/// }
/// ```
///
/// Can also add a block to decide whether to continue searching for some matches:
///
/// ```rust,ignore
/// impl<'a> Lexer<'a> {
///   fn eat_stuff(&mut self) -> bool {
///     let matched_byte = byte_search! {
///       lexer: self,
///       table: NOT_STUFF_TABLE,
///       continue_if: (matched_byte, pos) {
///         // Matching byte found. Decide whether it's really a match.
///         // Return `true` to continue searching, or `false` to end search.
///         // NB: `lexer.source` has NOT been updated at this point.
///         if matched_byte == 0xE2 {
///           // Only match a specific Unicode char (in this case 0xE2, 0x80, 0xA8)
///           // NB: We don't need to check if `pos` is at EOF here, as 0xE2 is always 1st byte
///           // of a 3-byte Unicode char, but if matching an ASCII char, would need to make sure
///           // don't read out of bounds.
///           unsafe { pos.add(1).read2() != [0x80, 0xA8] }
///         } else {
///           // End search for all other possibilities
///           false
///         }
///       },
///       handle_eof: {
///         // No bytes from start position to end of source matched the table.
///         // `lexer.source` is now positioned at EOF.
///         return false;
///       },
///     };
///
///     // Matching byte has been found.
///     // `matched_byte` is `u8` value of first byte which matched the table.
///     // `lexer.source` is now positioned on first matching byte.
///     // Handle the next matching byte (deal with any special cases).
///     matched_byte == b'X'
///   }
/// }
/// ```
///
/// # SAFETY
///
/// This macro consumes bytes from `lexer.source` according to the provided `SafeByteMatchTable`.
///
/// The `start` position must be on a UTF-8 character boundary. The overloads without an explicit
/// `start` use the current position of `lexer.source`.
///
/// Using `byte_search!` with the provided `SafeByteMatchTable` is guaranteed to leave
/// `lexer.source` positioned on a UTF-8 character boundary when entering `handle_match`, which
/// makes the internal calls to `Source`'s unsafe methods sound.
macro_rules! byte_search {
    // Simple version.
    // `start` is calculated from current position of `lexer.source`.
    (
        lexer: $lexer:ident,
        table: $table:ident,
        handle_eof: $eof_handler:expr,
    ) => {{
        let start = $lexer.source.position();
        byte_search! {
            lexer: $lexer,
            table: $table,
            start: start,
            continue_if: (byte, pos) false,
            handle_eof: $eof_handler,
        }
    }};

    // With `continue_if`.
    // `start` is calculated from current position of `lexer.source`.
    (
        lexer: $lexer:ident,
        table: $table:ident,
        continue_if: ($byte:ident, $pos:ident) $should_continue:expr,
        handle_eof: $eof_handler:expr,
    ) => {{
        let start = $lexer.source.position();
        byte_search! {
            lexer: $lexer,
            table: $table,
            start: start,
            continue_if: ($byte, $pos) $should_continue,
            handle_eof: $eof_handler,
        }
    }};

    // With provided `start` position
    (
        lexer: $lexer:ident,
        table: $table:ident,
        start: $start:ident,
        handle_eof: $eof_handler:expr,
    ) => {
        byte_search! {
            lexer: $lexer,
            table: $table,
            start: $start,
            continue_if: (byte, pos) false,
            handle_eof: $eof_handler,
        }
    };

    // Actual implementation - with both `start` and `continue_if`
    (
        lexer: $lexer:ident,
        table: $table:ident,
        start: $start:ident,
        continue_if: ($byte:ident, $pos:ident) $should_continue:expr,
        handle_eof: $eof_handler:expr,
    ) => {{
        // SAFETY:
        // The caller must provide `$start` on a UTF-8 character boundary. Given that precondition,
        // `$table` is always a `SafeByteMatchTable`, so it's guaranteed that `lexer.source` will be
        // positioned on a UTF-8 character boundary before `handle_match` is called. Therefore we
        // can assume this is taken care of and wrap the calls to unsafe functions in this macro
        // with `unsafe {}`.
        #[allow(clippy::unnecessary_safety_comment, clippy::allow_attributes)]
        $table.use_table();

        let mut $pos = $start;
        // Silence warnings if macro called in unsafe code
        #[allow(unused_unsafe, clippy::unnecessary_safety_comment, clippy::allow_attributes)]
        'outer: loop {
            let $byte = if $pos.can_read_batch_from(&$lexer.source) {
                // Search a batch of `SEARCH_BATCH_SIZE` bytes.
                //
                // `'inner: loop {}` is not a real loop - it always exits on first turn.
                // Only using `loop {}` so that can use `break 'inner` to get out of it.
                // This allows complex logic of `$should_continue` and `$match_handler` to be
                // outside the `for` loop, keeping it as minimal as possible, to encourage
                // compiler to unroll it.
                //
                // SAFETY:
                // `$pos.can_read_batch_from(&$lexer.source)` check above ensures there are
                // at least `SEARCH_BATCH_SIZE` bytes remaining in `lexer.source`.
                // So `$pos.add()` in this loop cannot go out of bounds.
                let batch = unsafe { $pos.slice(crate::lexer::search::SEARCH_BATCH_SIZE) };
                if let Some(i) = $table.search_batch(batch) {
                    // SAFETY: `i < SEARCH_BATCH_SIZE`, so `$pos.add(i)` is in bounds.
                    $pos = unsafe { $pos.add(i) };
                    // SAFETY: `$pos` is in bounds of `lexer.source`.
                    unsafe { $pos.read() }
                } else {
                    // No match in batch - search next batch.
                    // SAFETY: Cannot go out of bounds (see above).
                    // Also see above about UTF-8 character boundaries invariant.
                    $pos = unsafe { $pos.add(crate::lexer::search::SEARCH_BATCH_SIZE) };
                    continue 'outer;
                }
            } else {
                // Not enough bytes remaining for a batch. Process byte-by-byte.
                // Same as above, `'inner: loop {}` is not a real loop here - always exits on first turn.
                'inner: loop {
                    // SAFETY: `$pos` is before or equal to end of source
                    let remaining = unsafe {
                        let remaining_len = $lexer.source.end().offset_from($pos);
                        $pos.slice(remaining_len)
                    };
                    for (i, &byte) in remaining.iter().enumerate() {
                        if $table.matches(byte) {
                            // SAFETY: `i` is less than number of bytes remaining after `$pos`,
                            // so `$pos + i` cannot be out of bounds
                            $pos = unsafe { $pos.add(i) };
                            break 'inner byte;
                        }
                    }

                    // EOF.
                    // Advance `lexer.source`'s position to end of file.
                    $lexer.source.advance_to_end();

                    // Avoid lint errors if `$eof_handler` contains `return` statement
                    #[allow(
                        unused_variables,
                        unreachable_code,
                        clippy::diverging_sub_expression,
                        clippy::allow_attributes
                    )]
                    {
                        let eof_ret = $eof_handler;
                        break 'outer eof_ret;
                    }
                }
            };

            // Found match. Check if should continue.
            if $should_continue {
                // Not a match after all - continue searching.
                // SAFETY: `pos` is not at end of source, so safe to advance 1 byte.
                // See above about UTF-8 character boundaries invariant.
                $pos = unsafe { $pos.add(1) };
                continue;
            }

            // Match confirmed.
            // Advance `lexer.source`'s position up to `$pos`, consuming unmatched bytes.
            // SAFETY: See above about UTF-8 character boundaries invariant.
            $lexer.source.set_position($pos);

            break $byte;
        }
    }};
}
pub(crate) use byte_search;

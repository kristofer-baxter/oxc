use std::arch::aarch64::*;

use crate::pipeline::chunk::{load128, mm16_nibble, mm64_neon, veq128};

#[inline]
pub unsafe fn find1_neon(src: *const u8, n: usize, mut i: usize, a: u8) -> usize {
    while i + 16 <= n {
        let m = mm16_nibble(veq128(load128(src, i), a));
        if m != 0 {
            return i + (m.trailing_zeros() >> 2) as usize;
        }
        i += 16;
    }
    while i < n {
        if *src.add(i) == a {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find2_neon(src: *const u8, n: usize, mut i: usize, a: u8, b: u8) -> usize {
    while i + 16 <= n {
        let v = load128(src, i);
        let m = mm16_nibble(vorrq_u8(veq128(v, a), veq128(v, b)));
        if m != 0 {
            return i + (m.trailing_zeros() >> 2) as usize;
        }
        i += 16;
    }
    while i < n {
        let c = *src.add(i);
        if c == a || c == b {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find3_neon(src: *const u8, n: usize, mut i: usize, a: u8, b: u8, c: u8) -> usize {
    while i + 16 <= n {
        let v = load128(src, i);
        let m = mm16_nibble(vorrq_u8(vorrq_u8(veq128(v, a), veq128(v, b)), veq128(v, c)));
        if m != 0 {
            return i + (m.trailing_zeros() >> 2) as usize;
        }
        i += 16;
    }
    while i < n {
        let ch = *src.add(i);
        if ch == a || ch == b || ch == c {
            return i;
        }
        i += 1;
    }
    n
}

#[inline]
pub unsafe fn find4_neon(
    src: *const u8,
    n: usize,
    mut i: usize,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
) -> usize {
    while i + 16 <= n {
        let v = load128(src, i);
        let m = mm16_nibble(vorrq_u8(
            vorrq_u8(veq128(v, a), veq128(v, b)),
            vorrq_u8(veq128(v, c), veq128(v, d)),
        ));
        if m != 0 {
            return i + (m.trailing_zeros() >> 2) as usize;
        }
        i += 16;
    }
    while i < n {
        let x = *src.add(i);
        if x == a || x == b || x == c || x == d {
            return i;
        }
        i += 1;
    }
    n
}

macro_rules! define_find_function_neon {
    ($(#[$attr:meta])* $name:ident: $($needle:expr),+ $(,)?) => {
        $(#[$attr])*
        #[inline]
        pub unsafe fn $name(src: *const u8, n: usize, mut i: usize) -> usize {
            use std::arch::aarch64::vorrq_u8;
            use crate::pipeline::{find::neon::vor128, chunk::{load128, veq128, mm16_nibble}};

            while i + 16 <= n {
                let v = load128(src, i);
                let m = mm16_nibble(vor128!($(veq128(v, $needle)),+));
                if m != 0 {
                    return i + (m.trailing_zeros() >> 2) as usize;
                }
                i += 16;
            }
            while i < n {
                let c = *src.add(i);
                if $(c == $needle)||+ {
                    return i;
                }
                i += 1;
            }
            n
        }
    };
}
pub(super) use define_find_function_neon;

macro_rules! vor128 {
    ($a:expr) => { $a };
    ($a:expr, $b:expr) => { vorrq_u8($a, $b) };
    ($a:expr, $b:expr, $($rest:expr),+) => {
        vorrq_u8(vor128!($a, $b), vor128!($($rest),+))
    };
}
pub(super) use vor128;

#[inline(always)]
unsafe fn bracket_mask16(v: uint8x16_t) -> uint8x16_t {
    // '(' (0x28) and ')' (0x29) differ only in bit 0: (v & 0xFE) == 0x28.
    // '[' (0x5B) and '{' (0x7B), and ']' (0x5D) and '}' (0x7D), differ only in bit 5.
    let parens = vceqq_u8(vandq_u8(v, vdupq_n_u8(0xFE)), vdupq_n_u8(b'('));
    let folded = vandq_u8(v, vdupq_n_u8(0xDF));
    let sq_curly = vorrq_u8(vceqq_u8(folded, vdupq_n_u8(b'[')), vceqq_u8(folded, vdupq_n_u8(b']')));
    vorrq_u8(parens, sq_curly)
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`): bit `i` for byte `base + i`.
#[inline]
pub fn bracket_bits_neon(src: &[u8], base: usize) -> u64 {
    let block = &src[base..base + 64];
    let p = block.as_ptr();
    // SAFETY: `block` holds 64 bytes, and this module is compiled only when `neon` is enabled.
    unsafe {
        let m0 = bracket_mask16(load128(p, 0));
        let m1 = bracket_mask16(load128(p, 16));
        let m2 = bracket_mask16(load128(p, 32));
        let m3 = bracket_mask16(load128(p, 48));
        mm64_neon(m0, m1, m2, m3)
    }
}

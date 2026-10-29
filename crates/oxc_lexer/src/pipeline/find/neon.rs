use std::arch::aarch64::*;

use crate::pipeline::chunk::{load128, mm16_nibble, mm64_neon, veq128};

/// First byte at or after `i` that is one of `needles`, or `n`.
#[inline]
pub unsafe fn find_neon<const K: usize>(
    src: *const u8,
    n: usize,
    mut i: usize,
    needles: [u8; K],
) -> usize {
    while i + 16 <= n {
        let m = mm16_nibble(any_of(load128(src, i), needles));
        if m != 0 {
            return i + (m.trailing_zeros() >> 2) as usize;
        }
        i += 16;
    }
    while i < n {
        if needles.contains(&*src.add(i)) {
            return i;
        }
        i += 1;
    }
    n
}

// The compares are OR-folded pairwise, level by level, so the tree stays shallow for 7 needles.
#[inline(always)]
unsafe fn any_of<const N: usize>(v: uint8x16_t, needles: [u8; N]) -> uint8x16_t {
    let mut m = needles.map(|c| veq128(v, c));
    let mut len = N;
    while len > 1 {
        let half = len / 2;
        for j in 0..half {
            m[j] = vorrq_u8(m[2 * j], m[2 * j + 1]);
        }
        if len % 2 == 1 {
            m[half] = m[len - 1];
        }
        len = half + len % 2;
    }
    m[0]
}

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

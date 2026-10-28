use std::arch::aarch64::*;

use crate::pipeline::chunk::{load128, mm16_nibble, veq128};

use super::common::lic_verify_at;

pub unsafe fn scan_block_comment_neon(
    src: *const u8,
    n: usize,
    mut i: usize,
) -> (usize, bool, i64) {
    let mut saw_nl = false;
    let mut lic_q: i64 = -1;
    while i + 16 <= n {
        let v = load128(src, i);
        let vn = load128(src, i + 1);
        let term = mm16_nibble(vandq_u8(veq128(v, b'*'), veq128(vn, b'/')));
        let nl = if saw_nl { 0 } else { mm16_nibble(vorrq_u8(veq128(v, b'\n'), veq128(v, b'\r'))) };
        let at = if lic_q < 0 { mm16_nibble(veq128(v, b'@')) } else { 0 };
        if term != 0 {
            let j = (term.trailing_zeros() >> 2) as usize;
            let bodymask: u64 = if j > 0 { (1u64 << (j * 4)) - 1 } else { 0 };
            saw_nl |= (nl & bodymask) != 0;
            if lic_q < 0 {
                let am = at & bodymask;
                if am != 0 {
                    lic_q = lic_first16(src, i, am);
                }
            }
            return (i + j + 1, saw_nl, lic_q);
        }
        saw_nl |= nl != 0;
        if lic_q < 0 && at != 0 {
            lic_q = lic_first16(src, i, at);
        }
        i += 16;
    }
    while i + 1 < n {
        let c = *src.add(i);
        if c == b'*' && *src.add(i + 1) == b'/' {
            return (i + 1, saw_nl, lic_q);
        }
        if c == b'\n' || c == b'\r' {
            saw_nl = true;
        }
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i) {
            lic_q = i as i64;
        }
        i += 1;
    }
    (n, saw_nl, lic_q)
}

pub unsafe fn scan_line_comment_neon(src: *const u8, n: usize, mut i: usize) -> (usize, i64) {
    let mut lic_q: i64 = -1;
    while i + 16 <= n {
        let v = load128(src, i);
        let term_v = vorrq_u8(vorrq_u8(veq128(v, b'\n'), veq128(v, b'\r')), veq128(v, 0xE2));
        let mut term = mm16_nibble(term_v);
        let at = if lic_q < 0 { mm16_nibble(veq128(v, b'@')) } else { 0 };
        while term != 0 {
            let t = (term.trailing_zeros() >> 2) as usize;
            let c = *src.add(i + t);
            if c == 0xE2
                && !(*src.add(i + t + 1) == 0x80
                    && (*src.add(i + t + 2) == 0xA8 || *src.add(i + t + 2) == 0xA9))
            {
                term &= !(0xFu64 << (t * 4));
                continue;
            }
            if lic_q < 0 {
                let am = at & if t > 0 { (1u64 << (t * 4)) - 1 } else { 0 };
                if am != 0 {
                    lic_q = lic_first16(src, i, am);
                }
            }
            return (i + t, lic_q);
        }
        if lic_q < 0 && at != 0 {
            lic_q = lic_first16(src, i, at);
        }
        i += 16;
    }
    while i < n {
        let c = *src.add(i);
        if c == b'\n' || c == b'\r' {
            return (i, lic_q);
        }
        if c == 0xE2
            && *src.add(i + 1) == 0x80
            && (*src.add(i + 2) == 0xA8 || *src.add(i + 2) == 0xA9)
        {
            return (i, lic_q);
        }
        if c == b'@' && lic_q < 0 && lic_verify_at(src, i) {
            lic_q = i as i64;
        }
        i += 1;
    }
    (n, lic_q)
}

#[inline(always)]
unsafe fn lic_first16(src: *const u8, base: usize, mut am: u64) -> i64 {
    while am != 0 {
        let idx = (am.trailing_zeros() >> 2) as usize;
        let q = base + idx;
        am &= !(0xFu64 << (idx * 4));
        if lic_verify_at(src, q) {
            return q as i64;
        }
    }
    -1
}

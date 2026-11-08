use std::ptr;

use oxc_simd::cpu_version;

/// Load 64 bits starting at byte `i` as a `u64`.
///
/// The load is unaligned, so neither `src` nor `i` has any alignment requirement.
///
/// # SAFETY
///
/// `src.add(i)` must be valid for reads of 8 bytes.
#[cpu_version(Generic)]
#[inline(always)]
pub unsafe fn load64(src: *const u8, i: usize) -> u64 {
    ptr::read_unaligned(src.add(i) as *const u64)
}

/// Compare every byte of chunk `x` against `b`.
///
/// Returns a `u64` where each of the 8 bytes is `0x80`
/// where the byte in `x` equals `b`, and `0x00` where it does not.
///
/// On a little-endian target, `trailing_zeros() >> 3` gives the offset of the first match.
/// This is the SWAR counterpart of `Vec256::eq_u8` followed by `Vec256::movemask`.
#[cpu_version(Generic)]
#[inline(always)]
pub fn eqm(x: u64, b: u8) -> u64 {
    let lo = 0x0101_0101_0101_0101u64;
    let low7 = 0x7F7F_7F7F_7F7F_7F7Fu64;
    let y = x ^ lo.wrapping_mul(b as u64);
    !((y & low7).wrapping_add(low7) | y | low7)
}

#[cfg(target_arch = "aarch64")]
mod primitives_neon {
    use std::arch::aarch64::*;

    /// Load 128 bits starting at byte `i` as a `uint8x16_t`.
    ///
    /// # SAFETY
    ///
    /// - `src.add(i)` must be valid for reads of 16 bytes.
    #[inline(always)]
    pub unsafe fn load128(src: *const u8, i: usize) -> uint8x16_t {
        vld1q_u8(src.add(i))
    }

    /// Compare every byte of chunk `v` against `c`.
    #[inline(always)]
    pub unsafe fn veq128(v: uint8x16_t, c: u8) -> uint8x16_t {
        vceqq_u8(v, vdupq_n_u8(c))
    }

    /// Gather a 4-bit-per-byte mask of 16 `0xFF`/`0x00` bytes into a `u64`.
    ///
    /// `trailing_zeros() >> 2` gives the byte offset `0..16` of the first match.
    #[inline(always)]
    pub unsafe fn mm16_nibble(v: uint8x16_t) -> u64 {
        let narrowed = vshrn_n_u16::<4>(vreinterpretq_u16_u8(v));
        vget_lane_u64::<0>(vreinterpret_u64_u8(narrowed))
    }

    /// Gather the top bit of each byte of 4 chunks `a..d` into a 64-bit mask.
    #[inline(always)]
    pub unsafe fn mm64_neon(a: uint8x16_t, b: uint8x16_t, c: uint8x16_t, d: uint8x16_t) -> u64 {
        const BITS: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];
        let m = vld1q_u8(BITS.as_ptr());
        let ab = vpaddq_u8(vandq_u8(a, m), vandq_u8(b, m));
        let cd = vpaddq_u8(vandq_u8(c, m), vandq_u8(d, m));
        let s = vpaddq_u8(ab, cd);
        vgetq_lane_u64::<0>(vreinterpretq_u64_u8(vpaddq_u8(s, s)))
    }
}

#[cfg(target_arch = "aarch64")]
pub(super) use primitives_neon::*;

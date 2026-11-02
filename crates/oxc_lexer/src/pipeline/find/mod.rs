use crate::cpu::{Cpu, cpu_multiversion, cpu_select};

#[cfg(target_arch = "x86_64")]
mod avx2;
#[cfg(target_arch = "x86_64")]
use avx2::{bracket_bits_avx2, find_avx2};

#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "aarch64")]
use neon::{bracket_bits_neon, find_neon};

mod generic;
use generic::{bracket_bits_generic, find_generic};

/// First byte at or after `i` that is one of `needles`, or `n`.
///
/// Needle order matters for speed. See the comment on `any_of` in `avx2.rs`.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn find<const K: usize>(
    src: *const u8,
    n: usize,
    i: usize,
    needles: [u8; K],
    cpu: impl Cpu,
) -> usize {
    cpu_select!(cpu, {
        Avx2(cpu) => find_avx2(src, n, i, needles, cpu),
        Neon(cpu) => find_neon(src, n, i, needles, cpu),
        Generic(_) => find_generic(src, n, i, needles),
    })
}

/// Bits of the 64 bytes at `base` that are brackets (`(){}[]`). Bit `i` is for byte `base + i`.
#[cpu_multiversion]
#[inline(always)]
pub(super) fn bracket_bits(src: &[u8], base: usize, cpu: impl Cpu) -> u64 {
    cpu_select!(cpu, {
        Avx2(cpu) => bracket_bits_avx2(src, base, cpu),
        Neon(cpu) => bracket_bits_neon(src, base, cpu),
        Generic(_) => bracket_bits_generic(src, base),
    })
}

#[cfg(test)]
mod tests {
    use super::bracket_bits;
    use crate::cpu::DynCpu;

    #[test]
    fn bracket_bits_matches_all_256_bytes() {
        let cpu = DynCpu::detect();
        let mut all = [0u8; 256];
        for (i, b) in all.iter_mut().enumerate() {
            #[expect(clippy::cast_possible_truncation)]
            {
                *b = i as u8;
            }
        }
        for chunk in 0..4 {
            let base = chunk * 64;
            let got = bracket_bits(&all, base, cpu);
            let mut expected = 0u64;
            for i in 0..64 {
                if matches!(all[base + i], b'(' | b')' | b'{' | b'}' | b'[' | b']') {
                    expected |= 1u64 << i;
                }
            }
            assert_eq!(got, expected, "mismatch in chunk {chunk}");
        }
    }
}

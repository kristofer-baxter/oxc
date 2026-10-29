#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
mod avx2;
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
pub(super) use avx2::{bracket_bits_avx2 as bracket_bits, find_avx2 as find};

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod neon;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(super) use neon::{bracket_bits_neon as bracket_bits, find_neon as find};

#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
mod generic;
#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
pub(super) use generic::{bracket_bits_generic as bracket_bits, find_generic as find};

#[cfg(test)]
mod tests {
    use super::bracket_bits;

    #[test]
    fn bracket_bits_matches_all_256_bytes() {
        let mut all = [0u8; 256];
        for (i, b) in all.iter_mut().enumerate() {
            #[expect(clippy::cast_possible_truncation)]
            {
                *b = i as u8;
            }
        }
        for chunk in 0..4 {
            let base = chunk * 64;
            let got = bracket_bits(&all, base);
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

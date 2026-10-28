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
use avx2::define_find_function_avx2 as define_find_function;
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi2",
    target_feature = "popcnt"
))]
pub(super) use avx2::{
    bracket_bits_avx2 as bracket_bits, find1_avx2 as find1, find2_avx2 as find2,
    find3_avx2 as find3, find4_avx2 as find4,
};

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
mod neon;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use neon::define_find_function_neon as define_find_function;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub(super) use neon::{
    bracket_bits_neon as bracket_bits, find1_neon as find1, find2_neon as find2,
    find3_neon as find3, find4_neon as find4,
};

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
use generic::define_find_function_generic as define_find_function;
#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "popcnt"
    ),
    all(target_arch = "aarch64", target_feature = "neon")
)))]
pub(super) use generic::{
    bracket_bits_generic as bracket_bits, find1_generic as find1, find2_generic as find2,
    find3_generic as find3, find4_generic as find4,
};

mod common;
pub(super) use common::{
    find_jsx_tag, find_jsx_text, find_line_terminator, find_opener, find_opener_jsx5,
    find_opener_jsx7, find_opener6, find_regex, find_tmpl, unicode_ws_len,
};

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

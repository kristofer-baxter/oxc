use crate::cpu::{Cpu, cpu_multiversion, cpu_select};

#[cfg(target_arch = "x86_64")]
mod avx2;
#[cfg(target_arch = "x86_64")]
use avx2::{scan_block_comment_avx2, scan_line_comment_avx2};

#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "aarch64")]
use neon::{scan_block_comment_neon, scan_line_comment_neon};

mod generic;
use generic::{scan_block_comment_generic, scan_line_comment_generic};

mod common;
pub(super) use common::{scan_ident_esc, scan_number, scan_quoted, scan_regex, scan_tmpl_text};

#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn scan_block_comment(
    src: *const u8,
    n: usize,
    i: usize,
    cpu: impl Cpu,
) -> (usize, bool, i64) {
    cpu_select!(cpu, {
        Avx2(cpu) => scan_block_comment_avx2(src, n, i, cpu),
        Neon(cpu) => scan_block_comment_neon(src, n, i, cpu),
        Generic(_) => scan_block_comment_generic(src, n, i),
    })
}

/// Scan a `//` line comment to its LineTerminator (LF, CR, or LS/PS),
/// tracking the first `@license` / `@preserve` position for comment metadata.
#[cpu_multiversion]
#[inline(always)]
pub(super) unsafe fn scan_line_comment(
    src: *const u8,
    n: usize,
    i: usize,
    cpu: impl Cpu,
) -> (usize, i64) {
    cpu_select!(cpu, {
        Avx2(cpu) => scan_line_comment_avx2(src, n, i, cpu),
        Neon(cpu) => scan_line_comment_neon(src, n, i, cpu),
        Generic(_) => scan_line_comment_generic(src, n, i),
    })
}

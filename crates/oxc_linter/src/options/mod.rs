use crate::{FrameworkFlags, fixer::FixKind};

mod allow_warn_deny;
mod filter;

pub use allow_warn_deny::AllowWarnDeny;
pub use filter::{InvalidFilterKind, LintFilter, LintFilterKind};

/// Subset of options used directly by the linter.
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct LintOptions {
    pub fix: FixKind,
    pub framework_hints: FrameworkFlags,
    pub report_unused_directive: Option<AllowWarnDeny>,
    // If true, the linter will create "ignore this section / line" fixes for all diagnostics
    pub with_ignore_fixes: bool,
    /// If true, skip executing `warn`-severity rules (used when `--quiet` is enabled without
    /// `--deny-warnings` or `--max-warnings`).
    pub deny_only: bool,
}

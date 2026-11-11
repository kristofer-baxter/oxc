use std::{
    borrow::Cow,
    ffi::OsStr,
    fs,
    hash::BuildHasherDefault,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use lazy_regex::regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_span::{GetSpan, Span, VALID_EXTENSIONS};
use oxc_str::JSStr;
use rustc_hash::FxHasher;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use smallvec::SmallVec;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_relative_packages_diagnostic(
    span: Span,
    proper_import: &str,
    import_path: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Relative import from another package is not allowed. Use `{proper_import}` instead of `{import_path}`"
    ))
    .with_help(format!("Import `{proper_import}` by package name instead of a relative path."))
    .with_label(span)
}

#[derive(Debug, Clone)]
struct NamedPackage {
    name: String,
    directory: PathBuf,
    manifest_mtime: Option<std::time::SystemTime>,
}

#[derive(Deserialize)]
struct PackageJsonName {
    name: Option<String>,
}

#[derive(Clone)]
struct CachedPackageEntry {
    package: Option<Arc<NamedPackage>>,
    local_pkg_mtime: Option<std::time::SystemTime>,
}

fn file_mtime(path: &Path) -> Option<std::time::SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

fn read_package_name(pkg_json_path: &Path) -> Option<(String, Option<std::time::SystemTime>)> {
    let mtime = file_mtime(pkg_json_path);
    let bytes = fs::read(pkg_json_path).ok()?;
    let parsed: PackageJsonName = serde_json::from_slice(&bytes).ok()?;
    let name = parsed.name.filter(|name| !name.is_empty())?;
    Some((name, mtime))
}

#[derive(Debug, Default, Clone)]
pub struct NoRelativePackages(Box<NoRelativePackagesState>);

struct NoRelativePackagesState {
    config: NoRelativePackagesConfig,
    ignore_regexes: Vec<Regex>,
    package_cache: papaya::HashMap<PathBuf, CachedPackageEntry, BuildHasherDefault<FxHasher>>,
    resolver: Resolver,
}

impl std::fmt::Debug for NoRelativePackagesState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoRelativePackagesState")
            .field("config", &self.config)
            .field("ignore_regexes", &self.ignore_regexes)
            .finish_non_exhaustive()
    }
}

impl Clone for NoRelativePackagesState {
    fn clone(&self) -> Self {
        Self::new(self.config.clone(), self.ignore_regexes.clone())
    }
}

impl Default for NoRelativePackagesState {
    fn default() -> Self {
        Self::new(NoRelativePackagesConfig::default(), Vec::new())
    }
}

impl NoRelativePackagesState {
    fn new(config: NoRelativePackagesConfig, ignore_regexes: Vec<Regex>) -> Self {
        let package_cache = papaya::HashMap::builder()
            .hasher(BuildHasherDefault::default())
            .resize_mode(papaya::ResizeMode::Blocking)
            .build();
        let resolver = Resolver::new(ResolveOptions {
            extensions: VALID_EXTENSIONS.iter().map(|ext| format!(".{ext}")).collect(),
            main_fields: vec!["module".into(), "main".into()],
            condition_names: vec![
                "module".into(),
                "import".into(),
                "require".into(),
                "node".into(),
                "default".into(),
            ],
            extension_alias: vec![
                (".js".into(), vec![".js".into(), ".ts".into(), ".tsx".into()]),
                (".jsx".into(), vec![".jsx".into(), ".tsx".into(), ".ts".into()]),
                (".mjs".into(), vec![".mjs".into(), ".mts".into()]),
                (".cjs".into(), vec![".cjs".into(), ".cts".into()]),
            ],
            ..ResolveOptions::default()
        });
        Self { config, ignore_regexes, package_cache, resolver }
    }

    fn is_cached_entry_fresh(dir: &Path, entry: &CachedPackageEntry) -> bool {
        let local_pkg = dir.join("package.json");
        if file_mtime(&local_pkg) != entry.local_pkg_mtime {
            return false;
        }
        if let Some(pkg) = &entry.package {
            let manifest_path = pkg.directory.join("package.json");
            if file_mtime(&manifest_path) != pkg.manifest_mtime {
                return false;
            }
        }
        true
    }

    fn find_named_package(&self, start_dir: &Path) -> Option<Arc<NamedPackage>> {
        let cache = self.package_cache.pin();
        if let Some(cached) = cache.get(start_dir)
            && Self::is_cached_entry_fresh(start_dir, cached)
        {
            return cached.package.clone();
        }

        let mut visited: SmallVec<[(PathBuf, Option<std::time::SystemTime>); 8]> = SmallVec::new();
        let mut current = Some(start_dir);
        let mut found = None;

        while let Some(dir) = current {
            if let Some(cached) = cache.get(dir)
                && Self::is_cached_entry_fresh(dir, cached)
            {
                found.clone_from(&cached.package);
                break;
            }
            let pkg_json_path = dir.join("package.json");
            let local_mtime = file_mtime(&pkg_json_path);
            visited.push((dir.to_path_buf(), local_mtime));
            if local_mtime.is_some()
                && let Some((name, manifest_mtime)) = read_package_name(&pkg_json_path)
            {
                found = Some(Arc::new(NamedPackage {
                    name,
                    directory: dir.to_path_buf(),
                    manifest_mtime,
                }));
                break;
            }
            current = dir.parent();
        }

        for (dir, local_pkg_mtime) in visited {
            cache.insert(dir, CachedPackageEntry { package: found.clone(), local_pkg_mtime });
        }

        found
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoRelativePackagesConfig {
    /// If set to `true`, dependency paths for ES module import statements will be checked.
    pub esmodule: bool,
    /// If set to `true`, dependency paths for CommonJS `require()` calls will be checked.
    pub commonjs: bool,
    /// If set to `true`, dependency paths for AMD `define()` / `require()` calls will be checked.
    pub amd: bool,
    /// List of regex patterns for import paths to ignore.
    pub ignore: Vec<String>,
}

impl Default for NoRelativePackagesConfig {
    fn default() -> Self {
        Self { esmodule: true, commonjs: false, amd: false, ignore: Vec::new() }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids importing packages through relative paths (`./` or `../`) when the target file
    /// belongs to a different package (has a different nearest `package.json` with a `"name"` field).
    ///
    /// ### Why is this bad?
    ///
    /// In monorepos (such as Yarn/pnpm/Lerna workspaces), importing another workspace package via
    /// a relative path bypasses the package's public entrypoint and `package.json` exports, coupling
    /// consumers to the other package's internal directory structure.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import foo from '../other-package';
    /// import bar from '../../packages/other-package/src/bar';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import foo from 'other-package';
    /// import bar from './internal-module';
    /// import baz from '../shared-within-same-package';
    /// ```
    NoRelativePackages,
    import,
    restriction,
    fix,
    config = NoRelativePackagesConfig,
    version = "1.93.0",
    short_description = "Forbid importing packages through relative paths.",
);

impl Rule for NoRelativePackages {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let config = DefaultRuleConfig::<NoRelativePackagesConfig>::from_value(value)?.into_inner();
        let mut ignore_regexes = Vec::with_capacity(config.ignore.len());
        for pat in &config.ignore {
            let re = Regex::new(pat).map_err(|err| {
                serde::de::Error::custom(format!("Invalid `ignore` regex `{pat}`: {err}"))
            })?;
            ignore_regexes.push(re);
        }
        Ok(Self(Box::new(NoRelativePackagesState::new(config, ignore_regexes))))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) if self.0.config.esmodule => {
                self.check_import(import_decl.source.value, import_decl.source.span, ctx);
            }
            AstKind::ExportFromDeclaration(export_decl) if self.0.config.esmodule => {
                self.check_import(export_decl.source.value, export_decl.source.span, ctx);
            }
            AstKind::ExportAllDeclaration(export_decl) if self.0.config.esmodule => {
                self.check_import(export_decl.source.value, export_decl.source.span, ctx);
            }
            AstKind::CallExpression(call_expr) if self.0.config.commonjs || self.0.config.amd => {
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };
                let func_name = ident.name.as_str();
                let count = call_expr.arguments.len();
                if matches!(func_name, "require" | "define") && count > 0 {
                    match &call_expr.arguments[0] {
                        Argument::StringLiteral(str_literal)
                            if count == 1 && func_name == "require" && self.0.config.commonjs =>
                        {
                            self.check_import(str_literal.value, str_literal.span, ctx);
                        }
                        Argument::ArrayExpression(arr_expr) if count == 2 && self.0.config.amd => {
                            for el in &arr_expr.elements {
                                if let Some(Expression::StringLiteral(literal)) = el.as_expression()
                                {
                                    self.check_import(literal.value, el.span(), ctx);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

impl NoRelativePackages {
    fn check_import(&self, import_js_str: JSStr<'_>, span: Span, ctx: &LintContext<'_>) {
        let Some(import_path) = import_js_str.as_str() else {
            return;
        };

        if !is_relative_import(import_path) {
            return;
        }

        if self.0.ignore_regexes.iter().any(|re| re.is_match(import_path)) {
            return;
        }

        let context_path = ctx.file_path();
        let Some(context_dir) = context_path.parent() else {
            return;
        };

        let Some(context_pkg) = self.0.find_named_package(context_dir) else {
            return;
        };

        let loaded_module = ctx.module_record().get_loaded_module(import_path);
        let resolved_import: Cow<'_, Path> = if let Some(loaded) = loaded_module.as_deref() {
            Cow::Borrowed(loaded.resolved_absolute_path.as_path())
        } else if let Ok(resolution) = self.0.resolver.resolve_file(context_path, import_path) {
            Cow::Owned(resolution.into_path_buf())
        } else {
            return;
        };

        let Some(import_dir) = resolved_import.parent() else {
            return;
        };

        if import_dir == context_dir {
            return;
        }

        let Some(import_pkg) = self.0.find_named_package(import_dir) else {
            return;
        };

        if import_pkg.name != context_pkg.name {
            let proper_import = compute_proper_import(
                &import_pkg,
                context_dir,
                import_path,
                resolved_import.as_ref(),
            );
            let replacement = serde_json::to_string(&proper_import)
                .unwrap_or_else(|_| format!("\"{proper_import}\""));
            ctx.diagnostic_with_fix(
                no_relative_packages_diagnostic(span, &proper_import, import_path),
                |fixer| fixer.replace(span, replacement),
            );
        }
    }
}

#[inline]
fn is_relative_import(path: &str) -> bool {
    path == "."
        || path == ".."
        || path.starts_with("./")
        || path.starts_with("../")
        || path.starts_with(".\\")
        || path.starts_with("..\\")
}

fn posix_basename(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed)
}

fn normalize_lexical_path(base: &Path, relative_specifier: &str) -> PathBuf {
    let mut normalized = PathBuf::new();
    for comp in base.components() {
        normalized.push(comp.as_os_str());
    }
    for segment in relative_specifier.split(['/', '\\']) {
        match segment {
            "" | "." => {}
            ".." => {
                normalized.pop();
            }
            part => normalized.push(part),
        }
    }
    normalized
}

fn compute_proper_import(
    import_pkg: &NamedPackage,
    context_dir: &Path,
    import_path: &str,
    resolved_import: &Path,
) -> String {
    let import_root = &import_pkg.directory;
    let requested_target = normalize_lexical_path(context_dir, import_path);
    if let Ok(rel_requested) = requested_target.strip_prefix(import_root) {
        let mut result = import_pkg.name.clone();
        for component in rel_requested.components() {
            if let Component::Normal(part) = component
                && let Some(part_str) = part.to_str()
                && !part_str.is_empty()
            {
                result.push('/');
                result.push_str(part_str);
            }
        }
        return result;
    }

    let import_base_name = posix_basename(import_path);
    let import_root_base_name = import_root.file_name().and_then(OsStr::to_str).unwrap_or("");
    let mut result = import_pkg.name.clone();
    if let Ok(proper_path) = resolved_import.strip_prefix(import_root)
        && let Some(proper_dir) = proper_path.parent()
    {
        for component in proper_dir.components() {
            if let Component::Normal(part) = component
                && let Some(part_str) = part.to_str()
                && !part_str.is_empty()
            {
                result.push('/');
                result.push_str(part_str);
            }
        }
    }
    if import_base_name != import_root_base_name
        && !import_base_name.is_empty()
        && import_base_name != "."
        && import_base_name != ".."
    {
        result.push('/');
        result.push_str(import_base_name);
    }
    result
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (r"import foo from './bar'", None),
        (r"import foo from '../bar'", None),
        (r"import foo from 'package-named'", None),
        (r"import foo from './package/index.js'", None),
        (r"import foo from './package-named'", Some(json!([{ "ignore": ["package-named"] }]))),
        (r"const foo = require('./package-named')", None),
    ];

    let fail = vec![
        (r"import foo from './package-named'", None),
        (r"import foo from './package-named/index.js'", None),
        (r"import foo from './package-scoped'", None),
        (r"export { foo } from './package-named'", None),
        (r"export * from './package-scoped'", None),
        (r"const foo = require('./package-named')", Some(json!([{ "commonjs": true }]))),
    ];

    let fix = vec![
        (r"import foo from './package-named'", r#"import foo from "package-named""#, None),
        (r"import foo from './package-scoped'", r#"import foo from "@scope/package-named""#, None),
    ];

    Tester::new(NoRelativePackages::NAME, NoRelativePackages::PLUGIN, pass, fail)
        .expect_fix(fix)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}

#[test]
fn test_compute_proper_import_directory_index() {
    let pkg = NamedPackage {
        name: "package-named".to_string(),
        directory: PathBuf::from("/workspace/packages/pkg"),
        manifest_mtime: None,
    };
    let context_dir = Path::new("/workspace/packages/app");
    let resolved_dir_index = Path::new("/workspace/packages/pkg/src/index.js");
    assert_eq!(
        compute_proper_import(
            &pkg,
            context_dir,
            "../pkg",
            Path::new("/workspace/packages/pkg/dist/index.js"),
        ),
        "package-named"
    );
    assert_eq!(
        compute_proper_import(&pkg, context_dir, "../pkg/src", resolved_dir_index),
        "package-named/src"
    );
    assert_eq!(
        compute_proper_import(&pkg, context_dir, "../pkg/src/index.js", resolved_dir_index),
        "package-named/src/index.js"
    );
    assert_eq!(
        compute_proper_import(
            &pkg,
            context_dir,
            "../pkg/src/pkg",
            Path::new("/workspace/packages/pkg/src/pkg.ts"),
        ),
        "package-named/src/pkg"
    );
}

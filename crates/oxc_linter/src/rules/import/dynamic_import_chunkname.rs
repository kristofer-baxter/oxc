use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_allocator::Allocator;
use oxc_ast::{AstKind, Comment, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::{DefaultRuleConfig, Rule},
};

const DEFAULT_WEBPACK_CHUNKNAME_FORMAT: &str = r"([0-9a-zA-Z-_/.]|\[(request|index)\])+";

static COMMENT_STYLE_REGEX: Lazy<Regex> = lazy_regex!(
    r#"^( ((webpackChunkName: .+)|((webpackPrefetch|webpackPreload): (true|false|-?[0-9]+))|(webpackIgnore: (true|false))|((webpackInclude|webpackExclude): /.*?/)|(webpackMode: ["'](lazy|lazy-once|eager|weak)["'])|(webpackExports: (['"]\w+['"]|\[(['"]\w+['"], *)+(['"]\w+['"]*)\]))),?)+ $"#
);

static EAGER_MODE_REGEX: Lazy<Regex> = lazy_regex!(r#"webpackMode: ["']eager["'],? "#);

static DEFAULT_CHUNK_SUBSTR_REGEX: Lazy<Regex> =
    lazy_regex!(r#"webpackChunkName: ["']([0-9a-zA-Z-_/.]|\[(request|index)\])+["'],? "#);

fn missing_leading_comment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("dynamic imports require a leading comment with the webpack chunkname")
        .with_label(span)
}

fn block_comment_style_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("dynamic imports require a /* foo */ style comment, not a // foo comment")
        .with_label(span)
}

fn padded_block_comment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("dynamic imports require a block comment padded with spaces - /* foo */")
        .with_label(span)
}

fn valid_webpack_syntax_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(r#"dynamic imports require a "webpack" comment with valid syntax"#)
        .with_label(span)
}

fn eager_mode_unneeded_chunkname_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("dynamic imports using eager mode do not need a webpackChunkName")
        .with_label(span)
}

fn chunkname_format_diagnostic(span: Span, chunk_substr_format: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "dynamic imports require a leading comment in the form /*{chunk_substr_format}*/"
    ))
    .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct DynamicImportChunknameConfig {
    /// List of additional function names to treat as dynamic imports.
    pub import_functions: Vec<String>,
    /// Allow dynamic imports without any leading comments.
    pub allow_empty: bool,
    /// Regular expression pattern that `webpackChunkName` must match.
    pub webpack_chunkname_format: String,
}

impl Default for DynamicImportChunknameConfig {
    fn default() -> Self {
        Self {
            import_functions: Vec::new(),
            allow_empty: false,
            webpack_chunkname_format: DEFAULT_WEBPACK_CHUNKNAME_FORMAT.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DynamicImportChunkname(Box<DynamicImportChunknameState>);

#[derive(Debug, Clone)]
struct DynamicImportChunknameState {
    config: DynamicImportChunknameConfig,
    chunk_substr_format: String,
    chunk_substr_regex: Option<Regex>,
}

impl Default for DynamicImportChunknameState {
    fn default() -> Self {
        let config = DynamicImportChunknameConfig::default();
        let chunk_substr_format =
            format!("webpackChunkName: [\"']{}[\"'],? ", config.webpack_chunkname_format);
        Self { config, chunk_substr_format, chunk_substr_regex: None }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a leading comment with `webpackChunkName` for dynamic `import()` expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Unnamed dynamic imports produce anonymous numeric chunk filenames in Webpack/Rspack builds,
    /// making bundle analysis, caching, and debugging harder.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import('./foo');
    /// import(/* webpackChunkName: "" */ './foo');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// import(/* webpackChunkName: "my-chunk" */ './foo');
    /// import(/* webpackMode: "eager" */ './foo');
    /// ```
    DynamicImportChunkname,
    import,
    style,
    suggestion,
    config = DynamicImportChunknameConfig,
    version = "1.93.0",
    short_description = "Enforce a leading comment with the webpackChunkName for dynamic imports.",
);

impl Rule for DynamicImportChunkname {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let config =
            DefaultRuleConfig::<DynamicImportChunknameConfig>::from_value(value)?.into_inner();
        let chunk_substr_format =
            format!("webpackChunkName: [\"']{}[\"'],? ", config.webpack_chunkname_format);
        let chunk_substr_regex = if config.webpack_chunkname_format
            == DEFAULT_WEBPACK_CHUNKNAME_FORMAT
        {
            None
        } else {
            let compiled_pattern =
                format!("webpackChunkName: [\"'](?:{})[\"'],? ", config.webpack_chunkname_format);
            Some(Regex::new(&compiled_pattern).map_err(|err| {
                serde::de::Error::custom(format!(
                    "Invalid `webpackChunknameFormat` regex `{}`: {err}",
                    config.webpack_chunkname_format
                ))
            })?)
        };
        Ok(Self(Box::new(DynamicImportChunknameState {
            config,
            chunk_substr_format,
            chunk_substr_regex,
        })))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportExpression(import_expr) => {
                self.check_dynamic_import(
                    import_expr.span,
                    import_expr.span.start + 6,
                    Some(import_expr.source.span().start),
                    ctx,
                );
            }
            AstKind::CallExpression(call_expr) => {
                if self.0.config.import_functions.is_empty() {
                    return;
                }
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };
                if !self
                    .0
                    .config
                    .import_functions
                    .iter()
                    .any(|fn_name| fn_name == ident.name.as_str())
                {
                    return;
                }
                let arg_start = call_expr.arguments.first().map(|arg| arg.span().start);
                self.check_dynamic_import(call_expr.span, ident.span.end, arg_start, ctx);
            }
            _ => {}
        }
    }
}

impl DynamicImportChunkname {
    fn chunk_regex(&self) -> &Regex {
        self.0.chunk_substr_regex.as_ref().unwrap_or(&DEFAULT_CHUNK_SUBSTR_REGEX)
    }

    fn check_dynamic_import(
        &self,
        node_span: Span,
        callee_end: u32,
        arg_start: Option<u32>,
        ctx: &LintContext<'_>,
    ) {
        let Some(arg_start) = arg_start else {
            return;
        };

        let comment_search_start = ctx
            .find_prev_token_within(callee_end, arg_start, "(")
            .map_or(callee_end, |offset| callee_end + offset + 1);
        let leading_comments: Vec<&Comment> =
            ctx.comments_range(comment_search_start..arg_start).collect();

        if leading_comments.is_empty() {
            if !self.0.config.allow_empty {
                ctx.diagnostic(missing_leading_comment_diagnostic(node_span));
            }
            return;
        }

        let source_text = ctx.source_text();
        let mut is_chunkname_present = false;
        let mut is_eager_mode_present = false;
        let chunk_regex = self.chunk_regex();

        for comment in &leading_comments {
            if !comment.is_block() {
                ctx.diagnostic(block_comment_style_diagnostic(node_span));
                return;
            }

            let value = comment.content_span().source_text(source_text);
            if !is_padded_block_comment(value) {
                ctx.diagnostic(padded_block_comment_diagnostic(node_span));
                return;
            }

            if !is_valid_js_object_body(value) || !COMMENT_STYLE_REGEX.is_match(value) {
                ctx.diagnostic(valid_webpack_syntax_diagnostic(node_span));
                return;
            }

            if EAGER_MODE_REGEX.is_match(value) {
                is_eager_mode_present = true;
            }

            if chunk_regex.is_match(value) {
                is_chunkname_present = true;
            }
        }

        if is_chunkname_present && is_eager_mode_present {
            let chunk_fix = build_remove_directive_fix(&leading_comments, chunk_regex, source_text);
            let eager_fix =
                build_remove_directive_fix(&leading_comments, &EAGER_MODE_REGEX, source_text);
            let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
            let mut suggestions: Vec<RuleFix> = Vec::with_capacity(2);
            if let Some((span, replacement)) = chunk_fix {
                suggestions
                    .push(fixer.replace(span, replacement).with_message("Remove webpackChunkName"));
            }
            if let Some((span, replacement)) = eager_fix {
                suggestions
                    .push(fixer.replace(span, replacement).with_message("Remove webpackMode"));
            }
            ctx.diagnostic_with_suggestions(
                eager_mode_unneeded_chunkname_diagnostic(node_span),
                suggestions,
            );
        }

        if !is_chunkname_present && !self.0.config.allow_empty && !is_eager_mode_present {
            ctx.diagnostic(chunkname_format_diagnostic(node_span, &self.0.chunk_substr_format));
        }
    }
}

fn is_padded_block_comment(value: &str) -> bool {
    let Some(inner) = value.strip_prefix(' ').and_then(|s| s.strip_suffix(' ')) else {
        return false;
    };
    let mut chars = inner.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let Some(last) = chars.next_back() else {
        return false;
    };
    !first.is_whitespace() && !last.is_whitespace()
}

fn is_valid_js_object_body(value: &str) -> bool {
    let allocator = Allocator::default();
    let expr_src = format!("({{{value}}})");
    Parser::new(&allocator, &expr_src, SourceType::mjs()).parse_expression().is_ok()
}

fn build_remove_directive_fix(
    leading_comments: &[&Comment],
    regex: &Regex,
    source_text: &str,
) -> Option<(Span, String)> {
    for comment in leading_comments {
        let value = comment.content_span().source_text(source_text);
        if regex.is_match(value) {
            let replaced = regex.replace(value, "");
            let trimmed = replaced.trim().trim_end_matches(',');
            if trimmed.is_empty() {
                return Some((comment.span, String::new()));
            }
            return Some((comment.span, format!("/* {trimmed} */")));
        }
    }
    None
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (r#"import(/* webpackChunkName: "my-chunk-name" */ './a')"#, None),
        (r#"import(/* webpackChunkName: "my-chunk-name", webpackPrefetch: true */ './a')"#, None),
        (r#"import(/* webpackMode: "eager" */ './a')"#, None),
        (
            r#"import(/* webpackIgnore: true */ /* webpackChunkName: "my-chunk-name" */ './a')"#,
            None,
        ),
        (r"import('./a')", Some(json!([{ "allowEmpty": true }]))),
        (
            r#"dynamicImport(/* webpackChunkName: "my-chunk-name" */ './a')"#,
            Some(json!([{ "importFunctions": ["dynamicImport"] }])),
        ),
        (
            r#"import(/* webpackChunkName: "Chunk_1" */ './a')"#,
            Some(json!([{ "webpackChunknameFormat": "[a-zA-Z0-9-._]+" }])),
        ),
    ];

    let fail = vec![
        (r"import('./a')", None),
        (r#"import(/* webpackChunkName: "" */ './a')"#, None),
        (
            r#"import(// webpackChunkName: "my-chunk-name"
            './a')"#,
            None,
        ),
        (r#"import(/*webpackChunkName: "my-chunk-name"*/ './a')"#, None),
        (r#"import(/* webpackChunkName: "my-chunk-name", webpackMode: "eager" */ './a')"#, None),
        (r"dynamicImport('./a')", Some(json!([{ "importFunctions": ["dynamicImport"] }]))),
        (r#"import /* webpackChunkName: "my-chunk-name" */ ('./a')"#, None),
    ];

    let fix = vec![(
        r#"import(/* webpackChunkName: "my-chunk-name", webpackMode: "eager" */ './a')"#,
        r#"import(/* webpackMode: "eager" */ './a')"#,
        None,
    )];

    Tester::new(DynamicImportChunkname::NAME, DynamicImportChunkname::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_invalid_chunkname_format_rejected_and_alternation_grouped() {
    use serde_json::json;

    let bad =
        DynamicImportChunkname::from_configuration(json!([{ "webpackChunknameFormat": "(" }]));
    assert!(bad.is_err(), "invalid webpackChunknameFormat regex must fail closed");

    let grouped = DynamicImportChunkname::from_configuration(json!([{
        "webpackChunknameFormat": "alpha|beta"
    }]))
    .expect("valid alternation format should compile");
    let re = grouped.chunk_regex();
    assert!(re.is_match(" webpackChunkName: \"alpha\" "));
    assert!(re.is_match(" webpackChunkName: \"beta\" "));
    assert!(!re.is_match(" webpackChunkName: \"beta-unquoted"));
}

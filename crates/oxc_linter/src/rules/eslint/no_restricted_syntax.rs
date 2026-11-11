use lazy_regex::regex::{Regex, RegexBuilder};
use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, BindingPattern, ChainElement, Expression, FunctionType,
        ImportOrExportKind, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXElementName,
        JSXMemberExpressionObject, MethodDefinitionKind, ModuleExportName, PropertyKey,
        PropertyKind, SimpleAssignmentTarget, TSModuleReference,
    },
    ast_kind::{AST_TYPE_MAX, AstType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_restricted_syntax_diagnostic(span: Span, message: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string()).with_label(span)
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum RestrictedSyntaxItem {
    Selector(String),
    WithMessage {
        selector: String,
        #[serde(default)]
        message: Option<String>,
    },
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct NoRestrictedSyntaxConfig(Vec<RestrictedSyntaxItem>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Combinator {
    /// Direct child (`Parent > Child`)
    Child,
    /// Any ancestor (`Ancestor Descendant`)
    Descendant,
}

#[derive(Debug, Clone)]
enum AttrMatcher {
    Exists,
    Equals(AttrLiteral),
    NotEquals(AttrLiteral),
    Regex(Regex),
    LessThan(f64),
    LessEqual(f64),
    GreaterThan(f64),
    GreaterEqual(f64),
}

#[derive(Debug, Clone, PartialEq)]
enum AttrLiteral {
    Str(String),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
struct AttrPredicate {
    path: Vec<String>,
    matcher: AttrMatcher,
}

#[derive(Debug, Clone)]
enum NodeTypeConstraint {
    Any,
    AstTypes(AstTypesBitset),
    FunctionDeclaration,
    FunctionExpression,
    ClassDeclaration,
    ClassExpression,
}

impl NodeTypeConstraint {
    fn ast_types(&self) -> Option<AstTypesBitset> {
        match self {
            Self::Any => None,
            Self::AstTypes(bits) => Some(bits.clone()),
            Self::FunctionDeclaration | Self::FunctionExpression => {
                Some(AstTypesBitset::from_types(&[AstType::Function]))
            }
            Self::ClassDeclaration | Self::ClassExpression => {
                Some(AstTypesBitset::from_types(&[AstType::Class]))
            }
        }
    }

    fn matches(&self, kind: AstKind<'_>) -> bool {
        match self {
            Self::Any => true,
            Self::AstTypes(bits) => bits.has(kind.ty()),
            Self::FunctionDeclaration => {
                matches!(kind, AstKind::Function(f) if f.is_declaration() || f.r#type == FunctionType::TSDeclareFunction)
            }
            Self::FunctionExpression => {
                matches!(kind, AstKind::Function(f) if f.is_expression() || f.r#type == FunctionType::TSEmptyBodyFunctionExpression)
            }
            Self::ClassDeclaration => matches!(kind, AstKind::Class(c) if c.is_declaration()),
            Self::ClassExpression => matches!(kind, AstKind::Class(c) if c.is_expression()),
        }
    }
}

/// A single compound selector step matching one AST node (e.g. `CallExpression:matches(A, B)[callee.name='foo']`).
#[derive(Debug, Clone)]
struct CompoundSelector {
    type_constraint: NodeTypeConstraint,
    matches_alternatives: Vec<Vec<SelectorChain>>,
    not_alternatives: Vec<SelectorChain>,
    has_alternatives: Vec<Vec<SelectorChain>>,
    attributes: Vec<AttrPredicate>,
}

impl CompoundSelector {
    fn target_types(&self) -> Option<AstTypesBitset> {
        let mut base = self.type_constraint.ast_types();
        for group in &self.matches_alternatives {
            let mut group_union = AstTypesBitset::new();
            for alt in group {
                let alt_types = alt.target_types()?;
                for ty in &alt_types {
                    group_union.set(ty);
                }
            }
            base = Some(match base {
                Some(existing) => {
                    let mut intersected = AstTypesBitset::new();
                    for ty in &existing {
                        if group_union.has(ty) {
                            intersected.set(ty);
                        }
                    }
                    intersected
                }
                None => group_union,
            });
        }
        base
    }

    fn matches_node<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        let kind = node.kind();
        if !self.type_constraint.matches(kind) {
            return false;
        }
        for attr in &self.attributes {
            if !attr.matches(PropValue::AstKind(kind)) {
                return false;
            }
        }
        for group in &self.matches_alternatives {
            if !group.iter().any(|alt| alt.matches_node(node, ctx)) {
                return false;
            }
        }
        for neg in &self.not_alternatives {
            if neg.matches_node(node, ctx) {
                return false;
            }
        }
        for group in &self.has_alternatives {
            if !group.iter().any(|alt| alt.matches_within(node, ctx)) {
                return false;
            }
        }
        true
    }
}

/// A chain of compound selectors linked by combinators (`A > B C`).
/// Stored right-to-left: `head` matches the target node itself, and `ancestors` are checked upwards.
#[derive(Debug, Clone)]
struct SelectorChain {
    head: CompoundSelector,
    ancestors: Vec<(Combinator, CompoundSelector)>,
}

impl SelectorChain {
    fn target_types(&self) -> Option<AstTypesBitset> {
        self.head.target_types()
    }

    fn matches_node<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        if !self.head.matches_node(node, ctx) {
            return false;
        }
        if self.ancestors.is_empty() {
            return true;
        }
        Self::match_ancestors(&self.ancestors, node, ctx)
    }

    fn matches_within<'a>(&self, root: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
        let nodes = ctx.nodes();
        let root_id = root.id();
        let root_span = root.kind().span();
        for candidate in nodes {
            if candidate.id() == root_id {
                continue;
            }
            let cand_span = candidate.kind().span();
            if cand_span.start < root_span.start || cand_span.end > root_span.end {
                continue;
            }
            if !Self::is_descendant_of(candidate, root_id, nodes) {
                continue;
            }
            if !self.head.matches_node(candidate, ctx) {
                continue;
            }
            if self.ancestors.is_empty()
                || Self::match_ancestors_bounded(&self.ancestors, candidate, root_id, ctx)
            {
                return true;
            }
        }
        false
    }

    fn is_descendant_of(
        candidate: &AstNode<'_>,
        root_id: oxc_syntax::node::NodeId,
        nodes: &oxc_semantic::AstNodes<'_>,
    ) -> bool {
        let mut parent_id = nodes.parent_id(candidate.id());
        let mut prev_id = candidate.id();
        while parent_id != prev_id {
            if parent_id == root_id {
                return true;
            }
            prev_id = parent_id;
            parent_id = nodes.parent_id(parent_id);
        }
        false
    }

    fn match_ancestors_bounded<'a>(
        remaining: &[(Combinator, CompoundSelector)],
        current: &AstNode<'a>,
        root_id: oxc_syntax::node::NodeId,
        ctx: &LintContext<'a>,
    ) -> bool {
        let Some(((combinator, selector), rest)) = remaining.split_first() else {
            return true;
        };
        let nodes = ctx.nodes();
        match combinator {
            Combinator::Child => {
                let mut parent_id = nodes.parent_id(current.id());
                while parent_id != current.id() {
                    let parent = nodes.get_node(parent_id);
                    if matches!(
                        parent.kind(),
                        AstKind::ParenthesizedExpression(_) | AstKind::ChainExpression(_)
                    ) {
                        let next_id = nodes.parent_id(parent_id);
                        if next_id == parent_id {
                            return false;
                        }
                        parent_id = next_id;
                        continue;
                    }
                    if parent_id != root_id && !Self::is_descendant_of(parent, root_id, nodes) {
                        return false;
                    }
                    return selector.matches_node(parent, ctx)
                        && (rest.is_empty()
                            || Self::match_ancestors_bounded(rest, parent, root_id, ctx));
                }
                false
            }
            Combinator::Descendant => {
                let mut parent_id = nodes.parent_id(current.id());
                let mut prev_id = current.id();
                while parent_id != prev_id {
                    let parent = nodes.get_node(parent_id);
                    if selector.matches_node(parent, ctx)
                        && (rest.is_empty()
                            || Self::match_ancestors_bounded(rest, parent, root_id, ctx))
                    {
                        return true;
                    }
                    if parent_id == root_id {
                        break;
                    }
                    prev_id = parent_id;
                    parent_id = nodes.parent_id(parent_id);
                }
                false
            }
        }
    }

    fn match_ancestors<'a>(
        remaining: &[(Combinator, CompoundSelector)],
        current: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        let Some(((combinator, selector), rest)) = remaining.split_first() else {
            return true;
        };
        let nodes = ctx.nodes();
        match combinator {
            Combinator::Child => {
                let mut parent_id = nodes.parent_id(current.id());
                while parent_id != current.id() {
                    let parent = nodes.get_node(parent_id);
                    if matches!(
                        parent.kind(),
                        AstKind::ParenthesizedExpression(_) | AstKind::ChainExpression(_)
                    ) {
                        let next_id = nodes.parent_id(parent_id);
                        if next_id == parent_id {
                            return false;
                        }
                        parent_id = next_id;
                        continue;
                    }
                    return selector.matches_node(parent, ctx)
                        && (rest.is_empty() || Self::match_ancestors(rest, parent, ctx));
                }
                false
            }
            Combinator::Descendant => {
                let mut parent_id = nodes.parent_id(current.id());
                let mut prev_id = current.id();
                while parent_id != prev_id {
                    let parent = nodes.get_node(parent_id);
                    if selector.matches_node(parent, ctx)
                        && (rest.is_empty() || Self::match_ancestors(rest, parent, ctx))
                    {
                        return true;
                    }
                    prev_id = parent_id;
                    parent_id = nodes.parent_id(parent_id);
                }
                false
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CompiledPattern {
    chains: Vec<SelectorChain>,
    message: String,
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedSyntax(Box<NoRestrictedSyntaxState>);

#[derive(Debug, Clone)]
struct NoRestrictedSyntaxState {
    patterns: Vec<CompiledPattern>,
    target_types: AstTypesBitset,
    has_unbounded_selector: bool,
    /// Per-AstType indices into `patterns`: `(pattern_idx, chain_idx)`
    by_type: Box<[Vec<(usize, usize)>; AST_TYPE_MAX as usize + 1]>,
    unbounded_chains: Vec<(usize, usize)>,
}

impl Default for NoRestrictedSyntaxState {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            target_types: AstTypesBitset::new(),
            has_unbounded_selector: false,
            by_type: Box::new(std::array::from_fn(|_| Vec::new())),
            unbounded_chains: Vec::new(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows specified AST syntax patterns using ESQuery selectors.
    ///
    /// ### Why is this bad?
    ///
    /// Projects often want to ban specific language constructs, API call shapes, or AST patterns
    /// tailored to their architecture without writing a bespoke lint rule for each check.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /* eslint no-restricted-syntax: ["error", "WithStatement", "BinaryExpression[operator='in']"] */
    /// with (me) {
    ///   'bar' in foo;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /* eslint no-restricted-syntax: ["error", "WithStatement"] */
    /// me.bar();
    /// ```
    NoRestrictedSyntax,
    eslint,
    restriction,
    config = NoRestrictedSyntaxConfig,
    version = "1.93.0",
    short_description = "Disallow specified syntax.",
);

impl Rule for NoRestrictedSyntax {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        let mut raw_items = Vec::new();
        match value {
            Value::Null => {}
            Value::Array(arr) => {
                for elem in arr {
                    match elem {
                        Value::Array(nested) => {
                            for inner in nested {
                                raw_items
                                    .push(serde_json::from_value::<RestrictedSyntaxItem>(inner)?);
                            }
                        }
                        other => {
                            raw_items.push(serde_json::from_value::<RestrictedSyntaxItem>(other)?);
                        }
                    }
                }
            }
            other => {
                raw_items.push(serde_json::from_value::<RestrictedSyntaxItem>(other)?);
            }
        }

        let mut rule = NoRestrictedSyntaxState::default();
        for item in raw_items {
            let (raw_selector, custom_message) = match item {
                RestrictedSyntaxItem::Selector(s) => (s, None),
                RestrictedSyntaxItem::WithMessage { selector, message } => (selector, message),
            };
            let message =
                custom_message.unwrap_or_else(|| format!("Using '{raw_selector}' is not allowed."));
            let chains = parse_selector_list(&raw_selector).ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "Invalid or unsupported ESQuery selector `{raw_selector}`"
                ))
            })?;
            let pattern_idx = rule.patterns.len();
            for (chain_idx, chain) in chains.iter().enumerate() {
                if let Some(types) = chain.target_types() {
                    for ty in &types {
                        rule.target_types.set(ty);
                        rule.by_type[ty as usize].push((pattern_idx, chain_idx));
                    }
                } else {
                    rule.has_unbounded_selector = true;
                    rule.unbounded_chains.push((pattern_idx, chain_idx));
                }
            }
            rule.patterns.push(CompiledPattern { chains, message });
        }

        Ok(Self(Box::new(rule)))
    }

    fn should_run(&self, _ctx: &crate::context::ContextHost<'_>) -> bool {
        !self.0.patterns.is_empty()
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let state = &*self.0;
        if state.patterns.is_empty() {
            return;
        }

        let nodes = ctx.nodes();
        if !state.has_unbounded_selector && !nodes.contains_any(&state.target_types) {
            return;
        }

        // Track which patterns have already fired on the current node so a `:matches(A, B)` or
        // comma-separated selector list within one pattern only reports once per node.
        for node in nodes {
            let ty = node.kind().ty();
            let typed_candidates = &state.by_type[ty as usize];
            if typed_candidates.is_empty() && state.unbounded_chains.is_empty() {
                continue;
            }

            let mut last_reported_pattern: Option<usize> = None;
            for &(pattern_idx, chain_idx) in
                typed_candidates.iter().chain(state.unbounded_chains.iter())
            {
                if last_reported_pattern == Some(pattern_idx) {
                    continue;
                }
                let pattern = &state.patterns[pattern_idx];
                if pattern.chains[chain_idx].matches_node(node, ctx) {
                    last_reported_pattern = Some(pattern_idx);
                    ctx.diagnostic(no_restricted_syntax_diagnostic(
                        node.kind().span(),
                        &pattern.message,
                    ));
                }
            }
        }
    }
}

impl AttrPredicate {
    fn matches(&self, root: PropValue<'_>) -> bool {
        let mut current = root;
        for segment in &self.path {
            let Some(next) = current.get_prop(segment) else {
                return false;
            };
            current = next;
        }
        match &self.matcher {
            AttrMatcher::Exists => !matches!(current, PropValue::Null),
            AttrMatcher::Equals(lit) => current.equals_literal(lit),
            AttrMatcher::NotEquals(lit) => !current.equals_literal(lit),
            AttrMatcher::Regex(re) => current.as_str().is_some_and(|s| re.is_match(s)),
            AttrMatcher::LessThan(n) => current.as_number().is_some_and(|v| v < *n),
            AttrMatcher::LessEqual(n) => current.as_number().is_some_and(|v| v <= *n),
            AttrMatcher::GreaterThan(n) => current.as_number().is_some_and(|v| v > *n),
            AttrMatcher::GreaterEqual(n) => current.as_number().is_some_and(|v| v >= *n),
        }
    }
}

#[derive(Clone, Copy)]
enum PropValue<'a> {
    Str(&'a str),
    Bool(bool),
    Number(f64),
    Null,
    AstKind(AstKind<'a>),
    Expression(&'a Expression<'a>),
    Argument(&'a Argument<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    BindingPattern(&'a BindingPattern<'a>),
    PropertyKey(&'a PropertyKey<'a>),
    ModuleExportName(&'a ModuleExportName<'a>),
    TSModuleReference(&'a TSModuleReference<'a>),
    JSXElementName(&'a JSXElementName<'a>),
    JSXMemberExpressionObject(&'a JSXMemberExpressionObject<'a>),
    JSXAttributeName(&'a JSXAttributeName<'a>),
    JSXAttributeValue(&'a JSXAttributeValue<'a>),
    Arguments(&'a [Argument<'a>]),
    Expressions(&'a [Expression<'a>]),
    JSXAttributes(&'a [JSXAttributeItem<'a>]),
}

impl<'a> PropValue<'a> {
    fn as_str(self) -> Option<&'a str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_number(self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(n),
            _ => None,
        }
    }

    fn equals_literal(self, lit: &AttrLiteral) -> bool {
        match (self, lit) {
            (Self::Str(a), AttrLiteral::Str(b)) => a == b,
            (Self::Bool(a), AttrLiteral::Bool(b)) => a == *b,
            (Self::Number(a), AttrLiteral::Number(b)) => (a - *b).abs() < f64::EPSILON,
            (Self::Null, AttrLiteral::Null) => true,
            _ => false,
        }
    }

    fn estree_type(self) -> Option<&'static str> {
        match self {
            Self::AstKind(kind) => Some(ast_kind_estree_type(kind)),
            Self::Expression(expr) => Some(expression_estree_type(expr)),
            Self::Argument(arg) => match arg {
                Argument::SpreadElement(_) => Some("SpreadElement"),
                _ => arg.as_expression().map(expression_estree_type),
            },
            Self::AssignmentTarget(target) => match target {
                AssignmentTarget::ArrayAssignmentTarget(_) => Some("ArrayPattern"),
                AssignmentTarget::ObjectAssignmentTarget(_) => Some("ObjectPattern"),
                _ => target.as_simple_assignment_target().map(simple_target_estree_type),
            },
            Self::BindingPattern(pat) => Some(match pat {
                BindingPattern::BindingIdentifier(_) => "Identifier",
                BindingPattern::ObjectPattern(_) => "ObjectPattern",
                BindingPattern::ArrayPattern(_) => "ArrayPattern",
                BindingPattern::AssignmentPattern(_) => "AssignmentPattern",
            }),
            Self::PropertyKey(key) => match key {
                PropertyKey::StaticIdentifier(_) => Some("Identifier"),
                PropertyKey::PrivateIdentifier(_) => Some("PrivateIdentifier"),
                _ => key.as_expression().map(expression_estree_type),
            },
            Self::ModuleExportName(name) => Some(match name {
                ModuleExportName::IdentifierName(_) | ModuleExportName::IdentifierReference(_) => {
                    "Identifier"
                }
                ModuleExportName::StringLiteral(_) => "Literal",
            }),
            Self::TSModuleReference(r) => Some(match r {
                TSModuleReference::ExternalModuleReference(_) => "TSExternalModuleReference",
                TSModuleReference::IdentifierReference(_) => "Identifier",
                TSModuleReference::QualifiedName(_) => "TSQualifiedName",
            }),
            Self::JSXElementName(name) => Some(match name {
                JSXElementName::Identifier(_)
                | JSXElementName::IdentifierReference(_)
                | JSXElementName::ThisExpression(_) => "JSXIdentifier",
                JSXElementName::NamespacedName(_) => "JSXNamespacedName",
                JSXElementName::MemberExpression(_) => "JSXMemberExpression",
            }),
            Self::JSXMemberExpressionObject(obj) => Some(match obj {
                JSXMemberExpressionObject::IdentifierReference(_)
                | JSXMemberExpressionObject::ThisExpression(_) => "JSXIdentifier",
                JSXMemberExpressionObject::MemberExpression(_) => "JSXMemberExpression",
            }),
            Self::JSXAttributeName(name) => Some(match name {
                JSXAttributeName::Identifier(_) => "JSXIdentifier",
                JSXAttributeName::NamespacedName(_) => "JSXNamespacedName",
            }),
            Self::JSXAttributeValue(val) => Some(match val {
                JSXAttributeValue::StringLiteral(_) => "Literal",
                JSXAttributeValue::ExpressionContainer(_) => "JSXExpressionContainer",
                JSXAttributeValue::Element(_) => "JSXElement",
                JSXAttributeValue::Fragment(_) => "JSXFragment",
            }),
            Self::Str(_)
            | Self::Bool(_)
            | Self::Number(_)
            | Self::Null
            | Self::Arguments(_)
            | Self::Expressions(_)
            | Self::JSXAttributes(_) => None,
        }
    }

    fn get_prop(self, segment: &str) -> Option<PropValue<'a>> {
        if segment == "type" {
            return self.estree_type().map(PropValue::Str);
        }

        match self {
            Self::AstKind(kind) => get_ast_kind_prop(kind, segment),
            Self::Expression(expr) => get_expression_prop(expr, segment),
            Self::Argument(arg) => match arg {
                Argument::SpreadElement(spread) => match segment {
                    "argument" => Some(PropValue::Expression(&spread.argument)),
                    _ => None,
                },
                _ => get_expression_prop(arg.as_expression()?, segment),
            },
            Self::AssignmentTarget(target) => {
                if let Some(simple) = target.as_simple_assignment_target() {
                    get_simple_assignment_target_prop(simple, segment)
                } else {
                    None
                }
            }
            Self::BindingPattern(pat) => match pat {
                BindingPattern::BindingIdentifier(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                BindingPattern::AssignmentPattern(assign) => match segment {
                    "left" => Some(PropValue::BindingPattern(&assign.left)),
                    "right" => Some(PropValue::Expression(&assign.right)),
                    _ => None,
                },
                _ => None,
            },
            Self::PropertyKey(key) => match key {
                PropertyKey::StaticIdentifier(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                PropertyKey::PrivateIdentifier(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                _ => get_expression_prop(key.as_expression()?, segment),
            },
            Self::ModuleExportName(name) => match name {
                ModuleExportName::IdentifierName(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                ModuleExportName::IdentifierReference(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                ModuleExportName::StringLiteral(lit) => match segment {
                    "value" => lit.value.as_str().map(PropValue::Str),
                    _ => None,
                },
            },
            Self::TSModuleReference(r) => match r {
                TSModuleReference::ExternalModuleReference(ext) => match segment {
                    "expression" => {
                        Some(PropValue::AstKind(AstKind::StringLiteral(&ext.expression)))
                    }
                    _ => None,
                },
                TSModuleReference::IdentifierReference(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                TSModuleReference::QualifiedName(q) => {
                    get_ast_kind_prop(AstKind::TSQualifiedName(q), segment)
                }
            },
            Self::JSXElementName(name) => match name {
                JSXElementName::Identifier(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                JSXElementName::IdentifierReference(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                JSXElementName::ThisExpression(_) => match segment {
                    "name" => Some(PropValue::Str("this")),
                    _ => None,
                },
                JSXElementName::NamespacedName(ns) => {
                    get_ast_kind_prop(AstKind::JSXNamespacedName(ns), segment)
                }
                JSXElementName::MemberExpression(mem) => {
                    get_ast_kind_prop(AstKind::JSXMemberExpression(mem), segment)
                }
            },
            Self::JSXMemberExpressionObject(obj) => match obj {
                JSXMemberExpressionObject::IdentifierReference(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                JSXMemberExpressionObject::ThisExpression(_) => match segment {
                    "name" => Some(PropValue::Str("this")),
                    _ => None,
                },
                JSXMemberExpressionObject::MemberExpression(mem) => {
                    get_ast_kind_prop(AstKind::JSXMemberExpression(mem), segment)
                }
            },
            Self::JSXAttributeName(name) => match name {
                JSXAttributeName::Identifier(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                JSXAttributeName::NamespacedName(ns) => {
                    get_ast_kind_prop(AstKind::JSXNamespacedName(ns), segment)
                }
            },
            Self::JSXAttributeValue(val) => match val {
                JSXAttributeValue::StringLiteral(lit) => {
                    get_ast_kind_prop(AstKind::StringLiteral(lit), segment)
                }
                JSXAttributeValue::ExpressionContainer(c) => {
                    get_ast_kind_prop(AstKind::JSXExpressionContainer(c), segment)
                }
                JSXAttributeValue::Element(el) => {
                    get_ast_kind_prop(AstKind::JSXElement(el), segment)
                }
                JSXAttributeValue::Fragment(f) => {
                    get_ast_kind_prop(AstKind::JSXFragment(f), segment)
                }
            },
            Self::Arguments(args) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(args.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                args.get(idx).map(PropValue::Argument)
            }
            Self::Expressions(exprs) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(exprs.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                exprs.get(idx).map(PropValue::Expression)
            }
            Self::JSXAttributes(attrs) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(attrs.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                match attrs.get(idx)? {
                    JSXAttributeItem::Attribute(a) => {
                        Some(PropValue::AstKind(AstKind::JSXAttribute(a)))
                    }
                    JSXAttributeItem::SpreadAttribute(s) => {
                        Some(PropValue::AstKind(AstKind::JSXSpreadAttribute(s)))
                    }
                }
            }
            Self::Str(_) | Self::Bool(_) | Self::Number(_) | Self::Null => None,
        }
    }
}

fn get_simple_assignment_target_prop<'a>(
    target: &'a SimpleAssignmentTarget<'a>,
    segment: &str,
) -> Option<PropValue<'a>> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => match segment {
            "name" => Some(PropValue::Str(id.name.as_str())),
            _ => None,
        },
        SimpleAssignmentTarget::StaticMemberExpression(mem) => {
            get_ast_kind_prop(AstKind::StaticMemberExpression(mem), segment)
        }
        SimpleAssignmentTarget::ComputedMemberExpression(mem) => {
            get_ast_kind_prop(AstKind::ComputedMemberExpression(mem), segment)
        }
        SimpleAssignmentTarget::PrivateFieldExpression(mem) => {
            get_ast_kind_prop(AstKind::PrivateFieldExpression(mem), segment)
        }
        SimpleAssignmentTarget::TSAsExpression(ts) => get_expression_prop(&ts.expression, segment),
        SimpleAssignmentTarget::TSSatisfiesExpression(ts) => {
            get_expression_prop(&ts.expression, segment)
        }
        SimpleAssignmentTarget::TSNonNullExpression(ts) => {
            get_expression_prop(&ts.expression, segment)
        }
        SimpleAssignmentTarget::TSTypeAssertion(ts) => get_expression_prop(&ts.expression, segment),
    }
}

fn simple_target_estree_type(target: &SimpleAssignmentTarget<'_>) -> &'static str {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => "Identifier",
        SimpleAssignmentTarget::StaticMemberExpression(_)
        | SimpleAssignmentTarget::ComputedMemberExpression(_)
        | SimpleAssignmentTarget::PrivateFieldExpression(_) => "MemberExpression",
        SimpleAssignmentTarget::TSAsExpression(_) => "TSAsExpression",
        SimpleAssignmentTarget::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
        SimpleAssignmentTarget::TSNonNullExpression(_) => "TSNonNullExpression",
        SimpleAssignmentTarget::TSTypeAssertion(_) => "TSTypeAssertion",
    }
}

fn get_expression_prop<'a>(expr: &'a Expression<'a>, segment: &str) -> Option<PropValue<'a>> {
    let expr = expr.without_parentheses();
    match expr {
        Expression::Identifier(id) => get_ast_kind_prop(AstKind::IdentifierReference(id), segment),
        Expression::StaticMemberExpression(mem) => {
            get_ast_kind_prop(AstKind::StaticMemberExpression(mem), segment)
        }
        Expression::ComputedMemberExpression(mem) => {
            get_ast_kind_prop(AstKind::ComputedMemberExpression(mem), segment)
        }
        Expression::PrivateFieldExpression(mem) => {
            get_ast_kind_prop(AstKind::PrivateFieldExpression(mem), segment)
        }
        Expression::CallExpression(call) => {
            get_ast_kind_prop(AstKind::CallExpression(call), segment)
        }
        Expression::NewExpression(new_expr) => {
            get_ast_kind_prop(AstKind::NewExpression(new_expr), segment)
        }
        Expression::StringLiteral(lit) => get_ast_kind_prop(AstKind::StringLiteral(lit), segment),
        Expression::NumericLiteral(lit) => get_ast_kind_prop(AstKind::NumericLiteral(lit), segment),
        Expression::BooleanLiteral(lit) => get_ast_kind_prop(AstKind::BooleanLiteral(lit), segment),
        Expression::NullLiteral(lit) => get_ast_kind_prop(AstKind::NullLiteral(lit), segment),
        Expression::BigIntLiteral(lit) => get_ast_kind_prop(AstKind::BigIntLiteral(lit), segment),
        Expression::RegExpLiteral(lit) => get_ast_kind_prop(AstKind::RegExpLiteral(lit), segment),
        Expression::UnaryExpression(u) => get_ast_kind_prop(AstKind::UnaryExpression(u), segment),
        Expression::UpdateExpression(u) => get_ast_kind_prop(AstKind::UpdateExpression(u), segment),
        Expression::BinaryExpression(b) => get_ast_kind_prop(AstKind::BinaryExpression(b), segment),
        Expression::LogicalExpression(l) => {
            get_ast_kind_prop(AstKind::LogicalExpression(l), segment)
        }
        Expression::ConditionalExpression(c) => {
            get_ast_kind_prop(AstKind::ConditionalExpression(c), segment)
        }
        Expression::AssignmentExpression(a) => {
            get_ast_kind_prop(AstKind::AssignmentExpression(a), segment)
        }
        Expression::ImportExpression(imp) => {
            get_ast_kind_prop(AstKind::ImportExpression(imp), segment)
        }
        Expression::ArrowFunctionExpression(f) => {
            get_ast_kind_prop(AstKind::ArrowFunctionExpression(f), segment)
        }
        Expression::FunctionExpression(f) => get_ast_kind_prop(AstKind::Function(f), segment),
        Expression::ClassExpression(c) => get_ast_kind_prop(AstKind::Class(c), segment),
        Expression::AwaitExpression(a) => get_ast_kind_prop(AstKind::AwaitExpression(a), segment),
        Expression::YieldExpression(y) => get_ast_kind_prop(AstKind::YieldExpression(y), segment),
        Expression::TaggedTemplateExpression(t) => {
            get_ast_kind_prop(AstKind::TaggedTemplateExpression(t), segment)
        }
        Expression::TemplateLiteral(t) => get_ast_kind_prop(AstKind::TemplateLiteral(t), segment),
        Expression::ObjectExpression(o) => get_ast_kind_prop(AstKind::ObjectExpression(o), segment),
        Expression::ArrayExpression(a) => get_ast_kind_prop(AstKind::ArrayExpression(a), segment),
        Expression::ThisExpression(t) => get_ast_kind_prop(AstKind::ThisExpression(t), segment),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => {
                get_ast_kind_prop(AstKind::CallExpression(call), segment)
            }
            ChainElement::TSNonNullExpression(ts) => {
                get_ast_kind_prop(AstKind::TSNonNullExpression(ts), segment)
            }
            ChainElement::ComputedMemberExpression(mem) => {
                get_ast_kind_prop(AstKind::ComputedMemberExpression(mem), segment)
            }
            ChainElement::StaticMemberExpression(mem) => {
                get_ast_kind_prop(AstKind::StaticMemberExpression(mem), segment)
            }
            ChainElement::PrivateFieldExpression(mem) => {
                get_ast_kind_prop(AstKind::PrivateFieldExpression(mem), segment)
            }
        },
        Expression::TSAsExpression(ts) => get_ast_kind_prop(AstKind::TSAsExpression(ts), segment),
        Expression::TSSatisfiesExpression(ts) => {
            get_ast_kind_prop(AstKind::TSSatisfiesExpression(ts), segment)
        }
        Expression::TSNonNullExpression(ts) => {
            get_ast_kind_prop(AstKind::TSNonNullExpression(ts), segment)
        }
        Expression::TSTypeAssertion(ts) => get_ast_kind_prop(AstKind::TSTypeAssertion(ts), segment),
        Expression::JSXElement(el) => get_ast_kind_prop(AstKind::JSXElement(el), segment),
        Expression::JSXFragment(f) => get_ast_kind_prop(AstKind::JSXFragment(f), segment),
        _ => None,
    }
}

fn import_or_export_kind_str(kind: ImportOrExportKind) -> &'static str {
    match kind {
        ImportOrExportKind::Value => "value",
        ImportOrExportKind::Type => "type",
    }
}

fn get_ast_kind_prop<'a>(kind: AstKind<'a>, segment: &str) -> Option<PropValue<'a>> {
    match kind {
        AstKind::IdentifierReference(id) => match segment {
            "name" => Some(PropValue::Str(id.name.as_str())),
            _ => None,
        },
        AstKind::BindingIdentifier(id) => match segment {
            "name" => Some(PropValue::Str(id.name.as_str())),
            _ => None,
        },
        AstKind::IdentifierName(id) => match segment {
            "name" => Some(PropValue::Str(id.name.as_str())),
            _ => None,
        },
        AstKind::LabelIdentifier(id) => match segment {
            "name" => Some(PropValue::Str(id.name.as_str())),
            _ => None,
        },
        AstKind::PrivateIdentifier(id) => match segment {
            "name" => Some(PropValue::Str(id.name.as_str())),
            _ => None,
        },
        AstKind::JSXIdentifier(id) => match segment {
            "name" => Some(PropValue::Str(id.name.as_str())),
            _ => None,
        },
        AstKind::CallExpression(call) => match segment {
            "callee" => Some(PropValue::Expression(&call.callee)),
            "arguments" => Some(PropValue::Arguments(&call.arguments)),
            "optional" => Some(PropValue::Bool(call.optional)),
            _ => None,
        },
        AstKind::NewExpression(new_expr) => match segment {
            "callee" => Some(PropValue::Expression(&new_expr.callee)),
            "arguments" => Some(PropValue::Arguments(&new_expr.arguments)),
            _ => None,
        },
        AstKind::StaticMemberExpression(mem) => match segment {
            "object" => Some(PropValue::Expression(&mem.object)),
            "property" => Some(PropValue::AstKind(AstKind::IdentifierName(&mem.property))),
            "computed" => Some(PropValue::Bool(false)),
            "optional" => Some(PropValue::Bool(mem.optional)),
            _ => None,
        },
        AstKind::ComputedMemberExpression(mem) => match segment {
            "object" => Some(PropValue::Expression(&mem.object)),
            "property" => Some(PropValue::Expression(&mem.expression)),
            "computed" => Some(PropValue::Bool(true)),
            "optional" => Some(PropValue::Bool(mem.optional)),
            _ => None,
        },
        AstKind::PrivateFieldExpression(mem) => match segment {
            "object" => Some(PropValue::Expression(&mem.object)),
            "property" => Some(PropValue::AstKind(AstKind::PrivateIdentifier(&mem.field))),
            "computed" => Some(PropValue::Bool(false)),
            "optional" => Some(PropValue::Bool(mem.optional)),
            _ => None,
        },
        AstKind::ImportDeclaration(decl) => match segment {
            "source" => Some(PropValue::AstKind(AstKind::StringLiteral(&decl.source))),
            "importKind" => Some(PropValue::Str(import_or_export_kind_str(decl.import_kind))),
            _ => None,
        },
        AstKind::ExportNamedDeclaration(decl) => match segment {
            "source" => Some(PropValue::Null),
            "exportKind" => Some(PropValue::Str(import_or_export_kind_str(decl.export_kind))),
            _ => None,
        },
        AstKind::ExportFromDeclaration(decl) => match segment {
            "source" => Some(PropValue::AstKind(AstKind::StringLiteral(&decl.source))),
            "exportKind" => Some(PropValue::Str(import_or_export_kind_str(decl.export_kind))),
            _ => None,
        },
        AstKind::ExportAllDeclaration(decl) => match segment {
            "source" => Some(PropValue::AstKind(AstKind::StringLiteral(&decl.source))),
            "exportKind" => Some(PropValue::Str(import_or_export_kind_str(decl.export_kind))),
            "exported" => decl
                .exported
                .as_ref()
                .map_or(Some(PropValue::Null), |e| Some(PropValue::ModuleExportName(e))),
            _ => None,
        },
        AstKind::ImportExpression(imp) => match segment {
            "source" => Some(PropValue::Expression(&imp.source)),
            _ => None,
        },
        AstKind::TSImportEqualsDeclaration(decl) => match segment {
            "id" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&decl.id))),
            "moduleReference" => Some(PropValue::TSModuleReference(&decl.module_reference)),
            "importKind" => Some(PropValue::Str(import_or_export_kind_str(decl.import_kind))),
            _ => None,
        },
        AstKind::TSExternalModuleReference(ext) => match segment {
            "expression" => Some(PropValue::AstKind(AstKind::StringLiteral(&ext.expression))),
            _ => None,
        },
        AstKind::IfStatement(stmt) => match segment {
            "test" => Some(PropValue::Expression(&stmt.test)),
            _ => None,
        },
        AstKind::WhileStatement(stmt) => match segment {
            "test" => Some(PropValue::Expression(&stmt.test)),
            _ => None,
        },
        AstKind::DoWhileStatement(stmt) => match segment {
            "test" => Some(PropValue::Expression(&stmt.test)),
            _ => None,
        },
        AstKind::ForStatement(stmt) => match segment {
            "test" => {
                stmt.test.as_ref().map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e)))
            }
            _ => None,
        },
        AstKind::UnaryExpression(u) => match segment {
            "operator" => Some(PropValue::Str(u.operator.as_str())),
            "prefix" => Some(PropValue::Bool(true)),
            "argument" => Some(PropValue::Expression(&u.argument)),
            _ => None,
        },
        AstKind::UpdateExpression(u) => match segment {
            "operator" => Some(PropValue::Str(u.operator.as_str())),
            "prefix" => Some(PropValue::Bool(u.prefix)),
            _ => None,
        },
        AstKind::BinaryExpression(b) => match segment {
            "operator" => Some(PropValue::Str(b.operator.as_str())),
            "left" => Some(PropValue::Expression(&b.left)),
            "right" => Some(PropValue::Expression(&b.right)),
            _ => None,
        },
        AstKind::LogicalExpression(l) => match segment {
            "operator" => Some(PropValue::Str(l.operator.as_str())),
            "left" => Some(PropValue::Expression(&l.left)),
            "right" => Some(PropValue::Expression(&l.right)),
            _ => None,
        },
        AstKind::ConditionalExpression(c) => match segment {
            "test" => Some(PropValue::Expression(&c.test)),
            "consequent" => Some(PropValue::Expression(&c.consequent)),
            "alternate" => Some(PropValue::Expression(&c.alternate)),
            _ => None,
        },
        AstKind::VariableDeclarator(decl) => match segment {
            "id" => Some(PropValue::BindingPattern(&decl.id)),
            "init" => {
                decl.init.as_ref().map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e)))
            }
            _ => None,
        },
        AstKind::VariableDeclaration(decl) => match segment {
            "kind" => Some(PropValue::Str(decl.kind.as_str())),
            "declare" => Some(PropValue::Bool(decl.declare)),
            _ => None,
        },
        AstKind::AssignmentExpression(a) => match segment {
            "operator" => Some(PropValue::Str(a.operator.as_str())),
            "left" => Some(PropValue::AssignmentTarget(&a.left)),
            "right" => Some(PropValue::Expression(&a.right)),
            _ => None,
        },
        AstKind::StringLiteral(lit) => match segment {
            "value" => lit.value.as_str().map(PropValue::Str),
            "raw" => lit.raw.as_ref().map(|r| PropValue::Str(r.as_str())),
            _ => None,
        },
        AstKind::NumericLiteral(lit) => match segment {
            "value" => Some(PropValue::Number(lit.value)),
            "raw" => lit.raw.as_ref().map(|r| PropValue::Str(r.as_str())),
            _ => None,
        },
        AstKind::BooleanLiteral(lit) => match segment {
            "value" => Some(PropValue::Bool(lit.value)),
            _ => None,
        },
        AstKind::NullLiteral(_) => match segment {
            "value" => Some(PropValue::Null),
            _ => None,
        },
        AstKind::BigIntLiteral(lit) => match segment {
            "bigint" | "value" => Some(PropValue::Str(lit.value.as_str())),
            _ => None,
        },
        AstKind::RegExpLiteral(lit) => match segment {
            "pattern" => Some(PropValue::Str(lit.regex.pattern.text.as_str())),
            _ => None,
        },
        AstKind::Function(f) => match segment {
            "id" => f.id.as_ref().map_or(Some(PropValue::Null), |id| {
                Some(PropValue::AstKind(AstKind::BindingIdentifier(id)))
            }),
            "async" => Some(PropValue::Bool(f.r#async)),
            "generator" => Some(PropValue::Bool(f.generator)),
            "expression" => Some(PropValue::Bool(false)),
            _ => None,
        },
        AstKind::ArrowFunctionExpression(f) => match segment {
            "async" => Some(PropValue::Bool(f.r#async)),
            _ => None,
        },
        AstKind::Class(c) => match segment {
            "id" => c.id.as_ref().map_or(Some(PropValue::Null), |id| {
                Some(PropValue::AstKind(AstKind::BindingIdentifier(id)))
            }),
            "abstract" => Some(PropValue::Bool(c.r#abstract)),
            "declare" => Some(PropValue::Bool(c.declare)),
            _ => None,
        },
        AstKind::MethodDefinition(m) => match segment {
            "key" => Some(PropValue::PropertyKey(&m.key)),
            "static" => Some(PropValue::Bool(m.r#static)),
            "computed" => Some(PropValue::Bool(m.computed)),
            "kind" => Some(PropValue::Str(match m.kind {
                MethodDefinitionKind::Constructor => "constructor",
                MethodDefinitionKind::Method => "method",
                MethodDefinitionKind::Get => "get",
                MethodDefinitionKind::Set => "set",
            })),
            _ => None,
        },
        AstKind::PropertyDefinition(p) => match segment {
            "key" => Some(PropValue::PropertyKey(&p.key)),
            "value" => {
                p.value.as_ref().map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e)))
            }
            "static" => Some(PropValue::Bool(p.r#static)),
            "computed" => Some(PropValue::Bool(p.computed)),
            _ => None,
        },
        AstKind::ObjectProperty(p) => match segment {
            "key" => Some(PropValue::PropertyKey(&p.key)),
            "value" => Some(PropValue::Expression(&p.value)),
            "computed" => Some(PropValue::Bool(p.computed)),
            "shorthand" => Some(PropValue::Bool(p.shorthand)),
            "method" => Some(PropValue::Bool(p.method)),
            "kind" => Some(PropValue::Str(match p.kind {
                PropertyKind::Init => "init",
                PropertyKind::Get => "get",
                PropertyKind::Set => "set",
            })),
            _ => None,
        },
        AstKind::TaggedTemplateExpression(t) => match segment {
            "tag" => Some(PropValue::Expression(&t.tag)),
            "quasi" => Some(PropValue::AstKind(AstKind::TemplateLiteral(&t.quasi))),
            _ => None,
        },
        AstKind::TemplateLiteral(t) => match segment {
            "expressions" => Some(PropValue::Expressions(&t.expressions)),
            _ => None,
        },
        AstKind::AwaitExpression(a) => match segment {
            "argument" => Some(PropValue::Expression(&a.argument)),
            _ => None,
        },
        AstKind::YieldExpression(y) => match segment {
            "delegate" => Some(PropValue::Bool(y.delegate)),
            "argument" => y
                .argument
                .as_ref()
                .map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e))),
            _ => None,
        },
        AstKind::ReturnStatement(r) => match segment {
            "argument" => r
                .argument
                .as_ref()
                .map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e))),
            _ => None,
        },
        AstKind::ThrowStatement(t) => match segment {
            "argument" => Some(PropValue::Expression(&t.argument)),
            _ => None,
        },
        AstKind::ExpressionStatement(e) => match segment {
            "expression" => Some(PropValue::Expression(&e.expression)),
            _ => None,
        },
        AstKind::JSXElement(el) => match segment {
            "openingElement" => {
                Some(PropValue::AstKind(AstKind::JSXOpeningElement(&el.opening_element)))
            }
            _ => None,
        },
        AstKind::JSXOpeningElement(el) => match segment {
            "name" => Some(PropValue::JSXElementName(&el.name)),
            "attributes" => Some(PropValue::JSXAttributes(&el.attributes)),
            _ => None,
        },
        AstKind::JSXAttribute(attr) => match segment {
            "name" => Some(PropValue::JSXAttributeName(&attr.name)),
            "value" => attr
                .value
                .as_ref()
                .map_or(Some(PropValue::Null), |v| Some(PropValue::JSXAttributeValue(v))),
            _ => None,
        },
        AstKind::JSXMemberExpression(mem) => match segment {
            "object" => Some(PropValue::JSXMemberExpressionObject(&mem.object)),
            "property" => Some(PropValue::AstKind(AstKind::JSXIdentifier(&mem.property))),
            _ => None,
        },
        AstKind::JSXNamespacedName(ns) => match segment {
            "namespace" => Some(PropValue::AstKind(AstKind::JSXIdentifier(&ns.namespace))),
            "name" => Some(PropValue::AstKind(AstKind::JSXIdentifier(&ns.name))),
            _ => None,
        },
        AstKind::TSAsExpression(ts) => match segment {
            "expression" => Some(PropValue::Expression(&ts.expression)),
            _ => None,
        },
        AstKind::TSSatisfiesExpression(ts) => match segment {
            "expression" => Some(PropValue::Expression(&ts.expression)),
            _ => None,
        },
        AstKind::TSNonNullExpression(ts) => match segment {
            "expression" => Some(PropValue::Expression(&ts.expression)),
            _ => None,
        },
        AstKind::TSTypeAssertion(ts) => match segment {
            "expression" => Some(PropValue::Expression(&ts.expression)),
            _ => None,
        },
        AstKind::TSEnumDeclaration(decl) => match segment {
            "id" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&decl.id))),
            "const" => Some(PropValue::Bool(decl.r#const)),
            "declare" => Some(PropValue::Bool(decl.declare)),
            _ => None,
        },
        AstKind::TSInterfaceDeclaration(decl) => match segment {
            "id" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&decl.id))),
            "declare" => Some(PropValue::Bool(decl.declare)),
            _ => None,
        },
        AstKind::TSTypeAliasDeclaration(decl) => match segment {
            "id" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&decl.id))),
            "declare" => Some(PropValue::Bool(decl.declare)),
            _ => None,
        },
        AstKind::TSQualifiedName(q) => match segment {
            "right" => Some(PropValue::AstKind(AstKind::IdentifierName(&q.right))),
            _ => None,
        },
        _ => None,
    }
}

fn expression_estree_type(expr: &Expression<'_>) -> &'static str {
    match expr.without_parentheses() {
        Expression::Identifier(_) => "Identifier",
        Expression::StaticMemberExpression(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => "MemberExpression",
        Expression::CallExpression(_) => "CallExpression",
        Expression::NewExpression(_) => "NewExpression",
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => "Literal",
        Expression::UnaryExpression(_) => "UnaryExpression",
        Expression::UpdateExpression(_) => "UpdateExpression",
        Expression::BinaryExpression(_) | Expression::PrivateInExpression(_) => "BinaryExpression",
        Expression::LogicalExpression(_) => "LogicalExpression",
        Expression::ConditionalExpression(_) => "ConditionalExpression",
        Expression::AssignmentExpression(_) => "AssignmentExpression",
        Expression::SequenceExpression(_) => "SequenceExpression",
        Expression::ImportExpression(_) => "ImportExpression",
        Expression::ArrowFunctionExpression(_) => "ArrowFunctionExpression",
        Expression::FunctionExpression(_) => "FunctionExpression",
        Expression::ClassExpression(_) => "ClassExpression",
        Expression::AwaitExpression(_) => "AwaitExpression",
        Expression::YieldExpression(_) => "YieldExpression",
        Expression::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
        Expression::TemplateLiteral(_) => "TemplateLiteral",
        Expression::ObjectExpression(_) => "ObjectExpression",
        Expression::ArrayExpression(_) => "ArrayExpression",
        Expression::ThisExpression(_) => "ThisExpression",
        Expression::Super(_) => "Super",
        Expression::ChainExpression(_) => "ChainExpression",
        Expression::ImportMeta(_) | Expression::NewTarget(_) => "MetaProperty",
        Expression::JSXElement(_) => "JSXElement",
        Expression::JSXFragment(_) => "JSXFragment",
        Expression::TSAsExpression(_) => "TSAsExpression",
        Expression::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
        Expression::TSTypeAssertion(_) => "TSTypeAssertion",
        Expression::TSNonNullExpression(_) => "TSNonNullExpression",
        Expression::TSInstantiationExpression(_) => "TSInstantiationExpression",
        Expression::V8IntrinsicExpression(_) => "V8IntrinsicExpression",
        Expression::ParenthesizedExpression(_) => "ParenthesizedExpression",
    }
}

fn ast_kind_estree_type(kind: AstKind<'_>) -> &'static str {
    match kind {
        AstKind::Program(_) => "Program",
        AstKind::IdentifierName(_)
        | AstKind::IdentifierReference(_)
        | AstKind::BindingIdentifier(_)
        | AstKind::LabelIdentifier(_) => "Identifier",
        AstKind::ThisExpression(_) => "ThisExpression",
        AstKind::ArrayExpression(_) => "ArrayExpression",
        AstKind::ObjectExpression(_) => "ObjectExpression",
        AstKind::ObjectProperty(_)
        | AstKind::BindingProperty(_)
        | AstKind::AssignmentTargetPropertyIdentifier(_)
        | AstKind::AssignmentTargetPropertyProperty(_) => "Property",
        AstKind::TemplateLiteral(_) => "TemplateLiteral",
        AstKind::TaggedTemplateExpression(_) => "TaggedTemplateExpression",
        AstKind::TemplateElement(_) => "TemplateElement",
        AstKind::ComputedMemberExpression(_)
        | AstKind::StaticMemberExpression(_)
        | AstKind::PrivateFieldExpression(_) => "MemberExpression",
        AstKind::CallExpression(_) => "CallExpression",
        AstKind::NewExpression(_) => "NewExpression",
        AstKind::ImportMeta(_) | AstKind::NewTarget(_) => "MetaProperty",
        AstKind::SpreadElement(_) => "SpreadElement",
        AstKind::UpdateExpression(_) => "UpdateExpression",
        AstKind::UnaryExpression(_) => "UnaryExpression",
        AstKind::BinaryExpression(_) | AstKind::PrivateInExpression(_) => "BinaryExpression",
        AstKind::LogicalExpression(_) => "LogicalExpression",
        AstKind::ConditionalExpression(_) => "ConditionalExpression",
        AstKind::AssignmentExpression(_) => "AssignmentExpression",
        AstKind::SequenceExpression(_) => "SequenceExpression",
        AstKind::Super(_) => "Super",
        AstKind::AwaitExpression(_) => "AwaitExpression",
        AstKind::ChainExpression(_) => "ChainExpression",
        AstKind::BlockStatement(_) => "BlockStatement",
        AstKind::VariableDeclaration(_) => "VariableDeclaration",
        AstKind::VariableDeclarator(_) => "VariableDeclarator",
        AstKind::EmptyStatement(_) => "EmptyStatement",
        AstKind::ExpressionStatement(_) => "ExpressionStatement",
        AstKind::IfStatement(_) => "IfStatement",
        AstKind::DoWhileStatement(_) => "DoWhileStatement",
        AstKind::WhileStatement(_) => "WhileStatement",
        AstKind::ForStatement(_) => "ForStatement",
        AstKind::ForInStatement(_) => "ForInStatement",
        AstKind::ForOfStatement(_) => "ForOfStatement",
        AstKind::ContinueStatement(_) => "ContinueStatement",
        AstKind::BreakStatement(_) => "BreakStatement",
        AstKind::ReturnStatement(_) => "ReturnStatement",
        AstKind::WithStatement(_) => "WithStatement",
        AstKind::SwitchStatement(_) => "SwitchStatement",
        AstKind::SwitchCase(_) => "SwitchCase",
        AstKind::LabeledStatement(_) => "LabeledStatement",
        AstKind::ThrowStatement(_) => "ThrowStatement",
        AstKind::TryStatement(_) => "TryStatement",
        AstKind::CatchClause(_) => "CatchClause",
        AstKind::DebuggerStatement(_) => "DebuggerStatement",
        AstKind::AssignmentPattern(_) | AstKind::AssignmentTargetWithDefault(_) => {
            "AssignmentPattern"
        }
        AstKind::ObjectPattern(_) | AstKind::ObjectAssignmentTarget(_) => "ObjectPattern",
        AstKind::ArrayPattern(_) | AstKind::ArrayAssignmentTarget(_) => "ArrayPattern",
        AstKind::BindingRestElement(_)
        | AstKind::FormalParameterRest(_)
        | AstKind::AssignmentTargetRest(_) => "RestElement",
        AstKind::Function(f) => {
            if f.is_declaration() || f.r#type == FunctionType::TSDeclareFunction {
                "FunctionDeclaration"
            } else {
                "FunctionExpression"
            }
        }
        AstKind::ArrowFunctionExpression(_) => "ArrowFunctionExpression",
        AstKind::YieldExpression(_) => "YieldExpression",
        AstKind::Class(c) => {
            if c.is_declaration() {
                "ClassDeclaration"
            } else {
                "ClassExpression"
            }
        }
        AstKind::ClassBody(_) => "ClassBody",
        AstKind::MethodDefinition(_) => "MethodDefinition",
        AstKind::PropertyDefinition(_) => "PropertyDefinition",
        AstKind::PrivateIdentifier(_) => "PrivateIdentifier",
        AstKind::StaticBlock(_) => "StaticBlock",
        AstKind::AccessorProperty(_) => "AccessorProperty",
        AstKind::ImportExpression(_) => "ImportExpression",
        AstKind::ImportDeclaration(_) => "ImportDeclaration",
        AstKind::ImportSpecifier(_) => "ImportSpecifier",
        AstKind::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier",
        AstKind::ImportNamespaceSpecifier(_) => "ImportNamespaceSpecifier",
        AstKind::ExportNamedDeclaration(_) | AstKind::ExportFromDeclaration(_) => {
            "ExportNamedDeclaration"
        }
        AstKind::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
        AstKind::ExportAllDeclaration(_) => "ExportAllDeclaration",
        AstKind::ExportSpecifier(_) => "ExportSpecifier",
        AstKind::BooleanLiteral(_)
        | AstKind::NullLiteral(_)
        | AstKind::NumericLiteral(_)
        | AstKind::StringLiteral(_)
        | AstKind::BigIntLiteral(_)
        | AstKind::RegExpLiteral(_) => "Literal",
        AstKind::JSXElement(_) => "JSXElement",
        AstKind::JSXOpeningElement(_) => "JSXOpeningElement",
        AstKind::JSXClosingElement(_) => "JSXClosingElement",
        AstKind::JSXFragment(_) => "JSXFragment",
        AstKind::JSXNamespacedName(_) => "JSXNamespacedName",
        AstKind::JSXMemberExpression(_) => "JSXMemberExpression",
        AstKind::JSXExpressionContainer(_) => "JSXExpressionContainer",
        AstKind::JSXEmptyExpression(_) => "JSXEmptyExpression",
        AstKind::JSXAttribute(_) => "JSXAttribute",
        AstKind::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
        AstKind::JSXIdentifier(_) => "JSXIdentifier",
        AstKind::JSXText(_) => "JSXText",
        AstKind::TSEnumDeclaration(_) => "TSEnumDeclaration",
        AstKind::TSEnumMember(_) => "TSEnumMember",
        AstKind::TSTypeAnnotation(_) => "TSTypeAnnotation",
        AstKind::TSAnyKeyword(_) => "TSAnyKeyword",
        AstKind::TSStringKeyword(_) => "TSStringKeyword",
        AstKind::TSBooleanKeyword(_) => "TSBooleanKeyword",
        AstKind::TSNumberKeyword(_) => "TSNumberKeyword",
        AstKind::TSNeverKeyword(_) => "TSNeverKeyword",
        AstKind::TSUnknownKeyword(_) => "TSUnknownKeyword",
        AstKind::TSNullKeyword(_) => "TSNullKeyword",
        AstKind::TSUndefinedKeyword(_) => "TSUndefinedKeyword",
        AstKind::TSVoidKeyword(_) => "TSVoidKeyword",
        AstKind::TSTypeReference(_) => "TSTypeReference",
        AstKind::TSQualifiedName(_) => "TSQualifiedName",
        AstKind::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
        AstKind::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
        AstKind::TSAsExpression(_) => "TSAsExpression",
        AstKind::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
        AstKind::TSTypeAssertion(_) => "TSTypeAssertion",
        AstKind::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
        AstKind::TSExternalModuleReference(_) => "TSExternalModuleReference",
        AstKind::TSNonNullExpression(_) => "TSNonNullExpression",
        AstKind::Decorator(_) => "Decorator",
        AstKind::TSExportAssignment(_) => "TSExportAssignment",
        _ => "",
    }
}

fn parse_node_type_constraint(type_name: &str) -> Option<NodeTypeConstraint> {
    if type_name.is_empty() || type_name == "*" {
        return Some(NodeTypeConstraint::Any);
    }
    let types: &[AstType] = match type_name {
        "FunctionDeclaration" => return Some(NodeTypeConstraint::FunctionDeclaration),
        "FunctionExpression" => return Some(NodeTypeConstraint::FunctionExpression),
        "ClassDeclaration" => return Some(NodeTypeConstraint::ClassDeclaration),
        "ClassExpression" => return Some(NodeTypeConstraint::ClassExpression),
        "Program" => &[AstType::Program],
        "Identifier" => &[
            AstType::IdentifierReference,
            AstType::BindingIdentifier,
            AstType::IdentifierName,
            AstType::LabelIdentifier,
        ],
        "IdentifierReference" => &[AstType::IdentifierReference],
        "BindingIdentifier" => &[AstType::BindingIdentifier],
        "IdentifierName" => &[AstType::IdentifierName],
        "PrivateIdentifier" => &[AstType::PrivateIdentifier],
        "Literal" => &[
            AstType::StringLiteral,
            AstType::NumericLiteral,
            AstType::BooleanLiteral,
            AstType::NullLiteral,
            AstType::BigIntLiteral,
            AstType::RegExpLiteral,
        ],
        "StringLiteral" => &[AstType::StringLiteral],
        "NumericLiteral" => &[AstType::NumericLiteral],
        "BooleanLiteral" => &[AstType::BooleanLiteral],
        "NullLiteral" => &[AstType::NullLiteral],
        "BigIntLiteral" => &[AstType::BigIntLiteral],
        "RegExpLiteral" => &[AstType::RegExpLiteral],
        "MemberExpression" => &[
            AstType::StaticMemberExpression,
            AstType::ComputedMemberExpression,
            AstType::PrivateFieldExpression,
        ],
        "CallExpression" => &[AstType::CallExpression],
        "NewExpression" => &[AstType::NewExpression],
        "ImportDeclaration" => &[AstType::ImportDeclaration],
        "ImportSpecifier" => &[AstType::ImportSpecifier],
        "ImportDefaultSpecifier" => &[AstType::ImportDefaultSpecifier],
        "ImportNamespaceSpecifier" => &[AstType::ImportNamespaceSpecifier],
        "ExportNamedDeclaration" => {
            &[AstType::ExportNamedDeclaration, AstType::ExportFromDeclaration]
        }
        "ExportAllDeclaration" => &[AstType::ExportAllDeclaration],
        "ExportDefaultDeclaration" => &[AstType::ExportDefaultDeclaration],
        "ExportSpecifier" => &[AstType::ExportSpecifier],
        "ImportExpression" => &[AstType::ImportExpression],
        "TSImportEqualsDeclaration" => &[AstType::TSImportEqualsDeclaration],
        "TSExternalModuleReference" => &[AstType::TSExternalModuleReference],
        "IfStatement" => &[AstType::IfStatement],
        "WhileStatement" => &[AstType::WhileStatement],
        "DoWhileStatement" => &[AstType::DoWhileStatement],
        "ForStatement" => &[AstType::ForStatement],
        "ForInStatement" => &[AstType::ForInStatement],
        "ForOfStatement" => &[AstType::ForOfStatement],
        "UnaryExpression" => &[AstType::UnaryExpression],
        "UpdateExpression" => &[AstType::UpdateExpression],
        "LogicalExpression" => &[AstType::LogicalExpression],
        "ConditionalExpression" => &[AstType::ConditionalExpression],
        "BinaryExpression" => &[AstType::BinaryExpression, AstType::PrivateInExpression],
        "VariableDeclaration" => &[AstType::VariableDeclaration],
        "VariableDeclarator" => &[AstType::VariableDeclarator],
        "AssignmentExpression" => &[AstType::AssignmentExpression],
        "SequenceExpression" => &[AstType::SequenceExpression],
        "ArrowFunctionExpression" => &[AstType::ArrowFunctionExpression],
        "Function" => &[AstType::Function],
        "Class" => &[AstType::Class],
        "ClassBody" => &[AstType::ClassBody],
        "MethodDefinition" => &[AstType::MethodDefinition],
        "PropertyDefinition" => &[AstType::PropertyDefinition],
        "Property" | "ObjectProperty" => &[
            AstType::ObjectProperty,
            AstType::BindingProperty,
            AstType::AssignmentTargetPropertyIdentifier,
            AstType::AssignmentTargetPropertyProperty,
        ],
        "ObjectExpression" => &[AstType::ObjectExpression],
        "ArrayExpression" => &[AstType::ArrayExpression],
        "ObjectPattern" => &[AstType::ObjectPattern, AstType::ObjectAssignmentTarget],
        "ArrayPattern" => &[AstType::ArrayPattern, AstType::ArrayAssignmentTarget],
        "AssignmentPattern" => &[AstType::AssignmentPattern, AstType::AssignmentTargetWithDefault],
        "RestElement" => &[
            AstType::BindingRestElement,
            AstType::FormalParameterRest,
            AstType::AssignmentTargetRest,
        ],
        "SpreadElement" => &[AstType::SpreadElement],
        "TaggedTemplateExpression" => &[AstType::TaggedTemplateExpression],
        "TemplateLiteral" => &[AstType::TemplateLiteral],
        "TemplateElement" => &[AstType::TemplateElement],
        "AwaitExpression" => &[AstType::AwaitExpression],
        "YieldExpression" => &[AstType::YieldExpression],
        "ThisExpression" => &[AstType::ThisExpression],
        "Super" => &[AstType::Super],
        "ChainExpression" => &[AstType::ChainExpression],
        "BlockStatement" => &[AstType::BlockStatement],
        "ExpressionStatement" => &[AstType::ExpressionStatement],
        "ReturnStatement" => &[AstType::ReturnStatement],
        "ThrowStatement" => &[AstType::ThrowStatement],
        "TryStatement" => &[AstType::TryStatement],
        "CatchClause" => &[AstType::CatchClause],
        "SwitchStatement" => &[AstType::SwitchStatement],
        "SwitchCase" => &[AstType::SwitchCase],
        "BreakStatement" => &[AstType::BreakStatement],
        "ContinueStatement" => &[AstType::ContinueStatement],
        "LabeledStatement" => &[AstType::LabeledStatement],
        "WithStatement" => &[AstType::WithStatement],
        "DebuggerStatement" => &[AstType::DebuggerStatement],
        "EmptyStatement" => &[AstType::EmptyStatement],
        "JSXElement" => &[AstType::JSXElement],
        "JSXOpeningElement" => &[AstType::JSXOpeningElement],
        "JSXClosingElement" => &[AstType::JSXClosingElement],
        "JSXFragment" => &[AstType::JSXFragment],
        "JSXAttribute" => &[AstType::JSXAttribute],
        "JSXSpreadAttribute" => &[AstType::JSXSpreadAttribute],
        "JSXIdentifier" => &[AstType::JSXIdentifier],
        "JSXMemberExpression" => &[AstType::JSXMemberExpression],
        "JSXNamespacedName" => &[AstType::JSXNamespacedName],
        "JSXExpressionContainer" => &[AstType::JSXExpressionContainer],
        "JSXText" => &[AstType::JSXText],
        "TSEnumDeclaration" => &[AstType::TSEnumDeclaration],
        "TSEnumMember" => &[AstType::TSEnumMember],
        "TSInterfaceDeclaration" => &[AstType::TSInterfaceDeclaration],
        "TSTypeAliasDeclaration" => &[AstType::TSTypeAliasDeclaration],
        "TSTypeAssertion" => &[AstType::TSTypeAssertion],
        "TSAsExpression" => &[AstType::TSAsExpression],
        "TSSatisfiesExpression" => &[AstType::TSSatisfiesExpression],
        "TSNonNullExpression" => &[AstType::TSNonNullExpression],
        "TSAnyKeyword" => &[AstType::TSAnyKeyword],
        "TSTypeReference" => &[AstType::TSTypeReference],
        "TSQualifiedName" => &[AstType::TSQualifiedName],
        "TSExportAssignment" => &[AstType::TSExportAssignment],
        "Decorator" => &[AstType::Decorator],
        _ => return None,
    };
    Some(NodeTypeConstraint::AstTypes(AstTypesBitset::from_types(types)))
}

fn split_top_level(input: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_quote: Option<char> = None;
    let mut in_regex = false;
    let mut escaped = false;

    for (idx, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && (in_quote.is_some() || in_regex) {
            escaped = true;
            continue;
        }
        if let Some(q) = in_quote {
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        if in_regex {
            if ch == '/' {
                in_regex = false;
            }
            continue;
        }
        match ch {
            '\'' | '"' => in_quote = Some(ch),
            '/' if bracket_depth > 0 => in_regex = true,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            c if c == delimiter && paren_depth == 0 && bracket_depth == 0 => {
                parts.push(input[start..idx].trim());
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }
    let tail = input[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    parts
}

fn parse_selector_list(input: &str) -> Option<Vec<SelectorChain>> {
    let parts = split_top_level(input.trim(), ',');
    if parts.is_empty() {
        return None;
    }
    let mut chains = Vec::with_capacity(parts.len());
    for part in parts {
        chains.push(parse_selector_chain(part)?);
    }
    Some(chains)
}

fn parse_selector_chain(input: &str) -> Option<SelectorChain> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let mut steps: Vec<(Option<Combinator>, &str)> = Vec::new();
    let mut token_start: Option<usize> = None;
    let mut pending_combinator: Option<Combinator> = None;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_quote: Option<char> = None;
    let mut in_regex = false;
    let mut escaped = false;

    for (idx, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && (in_quote.is_some() || in_regex) {
            escaped = true;
            continue;
        }
        if let Some(q) = in_quote {
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        if in_regex {
            if ch == '/' {
                in_regex = false;
            }
            continue;
        }
        match ch {
            '\'' | '"' => {
                token_start.get_or_insert(idx);
                in_quote = Some(ch);
            }
            '/' if bracket_depth > 0 => {
                token_start.get_or_insert(idx);
                in_regex = true;
            }
            '(' => {
                token_start.get_or_insert(idx);
                paren_depth += 1;
            }
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => {
                token_start.get_or_insert(idx);
                bracket_depth += 1;
            }
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '>' if paren_depth == 0 && bracket_depth == 0 => {
                if let Some(start) = token_start.take() {
                    let comb = if steps.is_empty() {
                        None
                    } else {
                        Some(pending_combinator.take().unwrap_or(Combinator::Descendant))
                    };
                    steps.push((comb, &input[start..idx]));
                }
                pending_combinator = Some(Combinator::Child);
            }
            c if c.is_whitespace() && paren_depth == 0 && bracket_depth == 0 => {
                if let Some(start) = token_start.take() {
                    let comb = if steps.is_empty() {
                        None
                    } else {
                        Some(pending_combinator.take().unwrap_or(Combinator::Descendant))
                    };
                    steps.push((comb, &input[start..idx]));
                }
            }
            _ => {
                token_start.get_or_insert(idx);
            }
        }
    }

    if let Some(start) = token_start {
        let comb = if steps.is_empty() {
            None
        } else {
            Some(pending_combinator.take().unwrap_or(Combinator::Descendant))
        };
        steps.push((comb, &input[start..]));
    }

    if steps.is_empty() || pending_combinator.is_some() {
        return None;
    }

    let mut compounds: Vec<CompoundSelector> = Vec::with_capacity(steps.len());
    let mut combinators: Vec<Combinator> = Vec::with_capacity(steps.len().saturating_sub(1));
    for (idx, (comb, step_str)) in steps.into_iter().enumerate() {
        if idx > 0 {
            combinators.push(comb.unwrap_or(Combinator::Descendant));
        }
        compounds.push(parse_compound_selector(step_str)?);
    }

    let head = compounds.pop()?;
    let mut ancestors = Vec::with_capacity(compounds.len());
    while let (Some(comb), Some(comp)) = (combinators.pop(), compounds.pop()) {
        ancestors.push((comb, comp));
    }

    Some(SelectorChain { head, ancestors })
}

#[derive(Debug, Clone, Copy)]
enum PseudoKind {
    Matches,
    Not,
    Has,
}

fn parse_compound_selector(input: &str) -> Option<CompoundSelector> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let bytes = input.as_bytes();

    // Optional leading node type name or `*`
    let type_end = input.find(['[', ':']).unwrap_or(input.len());
    let type_name = input[..type_end].trim();
    let type_constraint = parse_node_type_constraint(type_name)?;
    let mut pos = type_end;

    let mut matches_alternatives = Vec::new();
    let mut not_alternatives = Vec::new();
    let mut has_alternatives = Vec::new();
    let mut attributes = Vec::new();

    while pos < bytes.len() {
        match bytes[pos] {
            b':' => {
                let rest = &input[pos + 1..];
                let (pseudo_kind, after_name) = if let Some(s) = rest.strip_prefix("matches(") {
                    (PseudoKind::Matches, s)
                } else if let Some(s) = rest.strip_prefix("is(") {
                    (PseudoKind::Matches, s)
                } else if let Some(s) = rest.strip_prefix("not(") {
                    (PseudoKind::Not, s)
                } else if let Some(s) = rest.strip_prefix("has(") {
                    (PseudoKind::Has, s)
                } else {
                    return None;
                };
                let prefix_len = rest.len() - after_name.len();
                let open_paren_end = pos + 1 + prefix_len;
                let close_offset = find_matching_delim(&input[open_paren_end..], '(', ')')?;
                let inner = &input[open_paren_end..open_paren_end + close_offset];
                let chains = parse_selector_list(inner)?;
                match pseudo_kind {
                    PseudoKind::Matches => matches_alternatives.push(chains),
                    PseudoKind::Not => not_alternatives.extend(chains),
                    PseudoKind::Has => has_alternatives.push(chains),
                }
                pos = open_paren_end + close_offset + 1;
            }
            b'[' => {
                let open_bracket_end = pos + 1;
                let close_offset = find_matching_delim(&input[open_bracket_end..], '[', ']')?;
                let inner = &input[open_bracket_end..open_bracket_end + close_offset];
                attributes.push(parse_attr_predicate(inner)?);
                pos = open_bracket_end + close_offset + 1;
            }
            c if c.is_ascii_whitespace() => {
                pos += 1;
            }
            _ => return None,
        }
    }

    Some(CompoundSelector {
        type_constraint,
        matches_alternatives,
        not_alternatives,
        has_alternatives,
        attributes,
    })
}

fn find_matching_delim(input: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 1usize;
    let mut in_quote: Option<char> = None;
    let mut in_regex = false;
    let mut escaped = false;

    for (idx, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && (in_quote.is_some() || in_regex) {
            escaped = true;
            continue;
        }
        if let Some(q) = in_quote {
            if ch == q {
                in_quote = None;
            }
            continue;
        }
        if in_regex {
            if ch == '/' {
                in_regex = false;
            }
            continue;
        }
        match ch {
            '\'' | '"' => in_quote = Some(ch),
            '/' => in_regex = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_attr_predicate(inner: &str) -> Option<AttrPredicate> {
    let inner = inner.trim();
    if inner.is_empty() {
        return None;
    }

    // Find operator outside quotes/regex: `!=`, `<=`, `>=`, `=`, `<`, `>`
    let mut op_pos_and_len: Option<(usize, usize)> = None;
    let bytes = inner.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'!' | b'<' | b'>' if i + 1 < bytes.len() && bytes[i + 1] == b'=' => {
                op_pos_and_len = Some((i, 2));
                break;
            }
            b'=' | b'<' | b'>' => {
                op_pos_and_len = Some((i, 1));
                break;
            }
            _ => i += 1,
        }
    }

    let Some((op_pos, op_len)) = op_pos_and_len else {
        let path: Vec<String> = inner
            .split('.')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect();
        if path.is_empty() {
            return None;
        }
        return Some(AttrPredicate { path, matcher: AttrMatcher::Exists });
    };

    let lhs = inner[..op_pos].trim();
    let op = &inner[op_pos..op_pos + op_len];
    let rhs = inner[op_pos + op_len..].trim();

    let path: Vec<String> =
        lhs.split('.').map(str::trim).filter(|s| !s.is_empty()).map(ToString::to_string).collect();
    if path.is_empty() || rhs.is_empty() {
        return None;
    }

    let matcher = match op {
        "=" => {
            if let Some(re) = parse_regex_literal(rhs) {
                AttrMatcher::Regex(re)
            } else {
                AttrMatcher::Equals(parse_attr_literal(rhs))
            }
        }
        "!=" => AttrMatcher::NotEquals(parse_attr_literal(rhs)),
        "<" => AttrMatcher::LessThan(rhs.parse::<f64>().ok()?),
        "<=" => AttrMatcher::LessEqual(rhs.parse::<f64>().ok()?),
        ">" => AttrMatcher::GreaterThan(rhs.parse::<f64>().ok()?),
        ">=" => AttrMatcher::GreaterEqual(rhs.parse::<f64>().ok()?),
        _ => return None,
    };

    Some(AttrPredicate { path, matcher })
}

fn parse_regex_literal(rhs: &str) -> Option<Regex> {
    let rest = rhs.strip_prefix('/')?;
    let mut escaped = false;
    let mut close_slash = None;
    for (idx, ch) in rest.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '/' {
            close_slash = Some(idx);
            break;
        }
    }
    let close_idx = close_slash?;
    let pattern = &rest[..close_idx];
    let flags = &rest[close_idx + 1..];
    RegexBuilder::new(pattern)
        .case_insensitive(flags.contains('i'))
        .multi_line(flags.contains('m'))
        .dot_matches_new_line(flags.contains('s'))
        .build()
        .ok()
}

fn parse_attr_literal(rhs: &str) -> AttrLiteral {
    if (rhs.starts_with('\'') && rhs.ends_with('\''))
        || (rhs.starts_with('"') && rhs.ends_with('"'))
    {
        let inner = &rhs[1..rhs.len() - 1];
        let mut unescaped = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(ch) = chars.next() {
            if ch == '\\'
                && let Some(next) = chars.next()
            {
                unescaped.push(next);
            } else {
                unescaped.push(ch);
            }
        }
        return AttrLiteral::Str(unescaped);
    }
    match rhs {
        "true" => AttrLiteral::Bool(true),
        "false" => AttrLiteral::Bool(false),
        "null" => AttrLiteral::Null,
        _ => {
            if let Ok(n) = rhs.parse::<f64>() {
                AttrLiteral::Number(n)
            } else {
                AttrLiteral::Str(rhs.to_string())
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (r"var me = 1;", None),
        (
            r"if (canUseDOM()) {}",
            Some(json!(["IfStatement[test.type='Identifier'][test.name='canUseDOM']"])),
        ),
        (
            r"if (!canUseDOM()) {}",
            Some(json!(["UnaryExpression[operator='!'] > Identifier[name='canUseDOM']"])),
        ),
        (
            r"import { foo } from '@1js/chat-ssr-sydney-connection';",
            Some(json!([
                {
                    "selector": ":matches(ImportDeclaration, ExportNamedDeclaration, ExportAllDeclaration)[source.value=/^@microsoft\\x2Fsignalr(\\x2F|$)/]",
                    "message": "Do not import SignalR."
                }
            ])),
        ),
        (
            r"const url = buildE2eUrl('/chat');",
            Some(json!([
                {
                    "selector": "Literal[value=/localhost:3000|127\\.0\\.0\\.1:3000/]",
                    "message": "Use buildE2eUrl()"
                }
            ])),
        ),
    ];

    let fail = vec![
        (r"with (me) {}", Some(json!(["WithStatement"]))),
        (
            r"if (canUseDOM) {}",
            Some(json!([
                {
                    "selector": "IfStatement[test.type='Identifier'][test.name='canUseDOM']",
                    "message": "canUseDOM must be called: use canUseDOM()"
                }
            ])),
        ),
        (
            r"if (!canUseDOM) {}",
            Some(json!([
                {
                    "selector": "UnaryExpression[operator='!'] > Identifier[name='canUseDOM']",
                    "message": "canUseDOM must be called: use !canUseDOM()"
                }
            ])),
        ),
        (
            r"const ok = canUseDOM && doWork();",
            Some(json!([
                {
                    "selector": "LogicalExpression > Identifier[name='canUseDOM']",
                    "message": "canUseDOM must be called: use canUseDOM() in logical expressions"
                }
            ])),
        ),
        (
            r"appStore.use.isBebopPresetEnabled();",
            Some(json!([
                {
                    "selector": "CallExpression[callee.type='MemberExpression'][callee.property.name='isBebopPresetEnabled'][callee.object.type='MemberExpression'][callee.object.property.name='use']",
                    "message": "Do not use appStore.use.isBebopPresetEnabled() directly."
                }
            ])),
        ),
        (
            r"const Comp = React.lazy(() => import('./Comp'));",
            Some(json!([
                {
                    "selector": "CallExpression[callee.type='MemberExpression'][callee.object.name='React'][callee.property.name='lazy']",
                    "message": "React.lazy() is banned on the SSR render path."
                }
            ])),
        ),
        (
            r"import { HubConnection } from '@microsoft/signalr/dist/esm';",
            Some(json!([
                {
                    "selector": ":matches(ImportDeclaration, ExportNamedDeclaration, ExportAllDeclaration)[source.value=/^@microsoft\\x2Fsignalr(\\x2F|$)/]",
                    "message": "Do not import SignalR."
                }
            ])),
        ),
        (
            r"export { HubConnection } from '@microsoft/signalr';",
            Some(json!([
                {
                    "selector": ":matches(ImportDeclaration, ExportNamedDeclaration, ExportAllDeclaration)[source.value=/^@microsoft\\x2Fsignalr(\\x2F|$)/]",
                    "message": "Do not import SignalR."
                }
            ])),
        ),
        (
            r"const ws = require('ws/lib/websocket');",
            Some(json!([
                {
                    "selector": "CallExpression[callee.name='require'][arguments.0.value=/^ws(\\x2F|$)/]",
                    "message": "Do not import ws."
                }
            ])),
        ),
        (
            r"import ws = require('ws');",
            Some(json!([
                {
                    "selector": "TSImportEqualsDeclaration[moduleReference.type='TSExternalModuleReference'][moduleReference.expression.value=/^ws(\\x2F|$)/]",
                    "message": "Do not import ws."
                }
            ])),
        ),
        (
            r"await page.waitForTimeout(500); const u = 'http://localhost:3000/test';",
            Some(json!([
                {
                    "selector": "CallExpression[callee.property.name='waitForTimeout']",
                    "message": "Avoid waitForTimeout"
                },
                {
                    "selector": "Literal[value=/localhost:3000|127\\.0\\.0\\.1:3000/]",
                    "message": "Use buildE2eUrl()"
                }
            ])),
        ),
    ];

    Tester::new(NoRestrictedSyntax::NAME, NoRestrictedSyntax::PLUGIN, pass, fail)
        .test_and_snapshot();
}

#[test]
fn test_has_pseudo_and_rejects_unsupported_selector() {
    use serde_json::json;

    let rule = NoRestrictedSyntax::from_configuration(json!([
        "CallExpression:has(Identifier[name='eval'])"
    ]))
    .expect(":has() selector should compile");
    assert_eq!(rule.0.patterns.len(), 1);

    let err =
        NoRestrictedSyntax::from_configuration(json!(["CallExpression:unknownPseudo(Identifier)"]));
    assert!(err.is_err(), "unsupported selector syntax must fail closed in from_configuration");
}

use lazy_regex::regex::{Regex, RegexBuilder};
use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrayExpressionElement, ArrowFunctionBody, AssignmentTarget,
        AssignmentTargetMaybeDefault, BindingPattern, BindingProperty, ChainElement, ClassElement,
        Declaration, ExportDefaultDeclarationKind, ExportSpecifier, Expression, ForStatementInit,
        ForStatementLeft, FormalParameters, FunctionType, ImportDeclarationSpecifier,
        ImportOrExportKind, JSXAttributeItem, JSXAttributeName, JSXAttributeValue, JSXChild,
        JSXElementName, JSXMemberExpressionObject, MethodDefinitionKind, ModuleExportName,
        ObjectPropertyKind, PropertyKey, PropertyKind, SimpleAssignmentTarget, Statement,
        SwitchCase, TSEnumMember, TSImportTypeQualifier, TSModuleReference, TSTypeName,
        TemplateElement, VariableDeclarator,
    },
    ast_kind::{AST_TYPE_MAX, AstType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smallvec::SmallVec;

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
            Self::Any => !is_estree_transparent_wrapper(kind),
            Self::AstTypes(bits) => match kind {
                AstKind::FormalParameter(param) => {
                    if param.has_modifier() {
                        bits.has(AstType::FormalParameter)
                    } else if param.initializer.is_some() {
                        bits.has(AstType::AssignmentPattern)
                    } else {
                        false
                    }
                }
                _ => bits.has(kind.ty()),
            },
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

#[inline]
fn is_estree_transparent_wrapper(kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::ParenthesizedExpression(_)
        | AstKind::FormalParameters(_)
        | AstKind::CatchParameter(_)
        | AstKind::Elision(_)
        | AstKind::WithClause(_) => true,
        AstKind::FormalParameter(param) => !param.has_modifier() && param.initializer.is_none(),
        _ => false,
    }
}

/// Returns the next ESTree-visible parent `AstNode` in the serialized ESTree graph,
/// skipping Oxc-internal wrapper nodes (`ParenthesizedExpression`, `FormalParameters`,
/// `CatchParameter`, and plain `FormalParameter` wrappers that have neither a modifier
/// nor a default initializer).
fn next_estree_parent<'b, 'a>(
    current_id: oxc_syntax::node::NodeId,
    nodes: &'b oxc_semantic::AstNodes<'a>,
) -> Option<&'b AstNode<'a>> {
    let mut id = current_id;
    loop {
        let parent_id = nodes.parent_id(id);
        if parent_id == id {
            return None;
        }
        let parent = nodes.get_node(parent_id);
        if is_estree_transparent_wrapper(parent.kind()) {
            id = parent_id;
            continue;
        }
        return Some(parent);
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

    fn has_any_has(&self) -> bool {
        if !self.has_alternatives.is_empty() {
            return true;
        }
        self.matches_alternatives.iter().flatten().any(SelectorChain::has_any_has)
            || self.not_alternatives.iter().any(SelectorChain::has_any_has)
    }

    fn matches_node<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        children_by_parent: Option<&[SmallVec<[oxc_syntax::node::NodeId; 4]>]>,
    ) -> bool {
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
            if !group.iter().any(|alt| alt.matches_node(node, ctx, children_by_parent)) {
                return false;
            }
        }
        for neg in &self.not_alternatives {
            if neg.matches_node(node, ctx, children_by_parent) {
                return false;
            }
        }
        for group in &self.has_alternatives {
            if !group.iter().any(|alt| alt.matches_within(node, ctx, children_by_parent)) {
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

    fn has_any_has(&self) -> bool {
        self.head.has_any_has() || self.ancestors.iter().any(|(_, c)| c.has_any_has())
    }

    fn matches_node<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
        children_by_parent: Option<&[SmallVec<[oxc_syntax::node::NodeId; 4]>]>,
    ) -> bool {
        if !self.head.matches_node(node, ctx, children_by_parent) {
            return false;
        }
        if self.ancestors.is_empty() {
            return true;
        }
        Self::match_ancestors(&self.ancestors, node, ctx, children_by_parent)
    }

    fn matches_within<'a>(
        &self,
        root: &AstNode<'a>,
        ctx: &LintContext<'a>,
        children_by_parent: Option<&[SmallVec<[oxc_syntax::node::NodeId; 4]>]>,
    ) -> bool {
        let nodes = ctx.nodes();
        let root_id = root.id();
        if let Some(children_map) = children_by_parent {
            let mut stack: SmallVec<[oxc_syntax::node::NodeId; 16]> = SmallVec::new();
            if let Some(direct) = children_map.get(root_id.index()) {
                stack.extend(direct.iter().copied());
            }
            while let Some(cand_id) = stack.pop() {
                let candidate = nodes.get_node(cand_id);
                if self.head.matches_node(candidate, ctx, children_by_parent)
                    && (self.ancestors.is_empty()
                        || Self::match_ancestors_bounded(
                            &self.ancestors,
                            candidate,
                            root_id,
                            ctx,
                            children_by_parent,
                        ))
                {
                    return true;
                }
                if let Some(children) = children_map.get(cand_id.index()) {
                    stack.extend(children.iter().copied());
                }
            }
            return false;
        }

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
            if !self.head.matches_node(candidate, ctx, None) {
                continue;
            }
            if self.ancestors.is_empty()
                || Self::match_ancestors_bounded(&self.ancestors, candidate, root_id, ctx, None)
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
        children_by_parent: Option<&[SmallVec<[oxc_syntax::node::NodeId; 4]>]>,
    ) -> bool {
        let Some(((combinator, selector), rest)) = remaining.split_first() else {
            return true;
        };
        let nodes = ctx.nodes();
        match combinator {
            Combinator::Child => {
                let Some(parent) = next_estree_parent(current.id(), nodes) else {
                    return false;
                };
                if parent.id() != root_id && !Self::is_descendant_of(parent, root_id, nodes) {
                    return false;
                }
                selector.matches_node(parent, ctx, children_by_parent)
                    && (rest.is_empty()
                        || Self::match_ancestors_bounded(
                            rest,
                            parent,
                            root_id,
                            ctx,
                            children_by_parent,
                        ))
            }
            Combinator::Descendant => {
                let mut cur_id = current.id();
                while let Some(parent) = next_estree_parent(cur_id, nodes) {
                    if parent.id() != root_id && !Self::is_descendant_of(parent, root_id, nodes) {
                        return false;
                    }
                    if selector.matches_node(parent, ctx, children_by_parent)
                        && (rest.is_empty()
                            || Self::match_ancestors_bounded(
                                rest,
                                parent,
                                root_id,
                                ctx,
                                children_by_parent,
                            ))
                    {
                        return true;
                    }
                    if parent.id() == root_id {
                        break;
                    }
                    cur_id = parent.id();
                }
                false
            }
        }
    }

    fn match_ancestors<'a>(
        remaining: &[(Combinator, CompoundSelector)],
        current: &AstNode<'a>,
        ctx: &LintContext<'a>,
        children_by_parent: Option<&[SmallVec<[oxc_syntax::node::NodeId; 4]>]>,
    ) -> bool {
        let Some(((combinator, selector), rest)) = remaining.split_first() else {
            return true;
        };
        let nodes = ctx.nodes();
        match combinator {
            Combinator::Child => {
                let Some(parent) = next_estree_parent(current.id(), nodes) else {
                    return false;
                };
                selector.matches_node(parent, ctx, children_by_parent)
                    && (rest.is_empty()
                        || Self::match_ancestors(rest, parent, ctx, children_by_parent))
            }
            Combinator::Descendant => {
                let mut cur_id = current.id();
                while let Some(parent) = next_estree_parent(cur_id, nodes) {
                    if selector.matches_node(parent, ctx, children_by_parent)
                        && (rest.is_empty()
                            || Self::match_ancestors(rest, parent, ctx, children_by_parent))
                    {
                        return true;
                    }
                    cur_id = parent.id();
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
    has_has_selector: bool,
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
            has_has_selector: false,
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
                if chain.has_any_has() {
                    rule.has_has_selector = true;
                }
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

        let children_map: Option<Vec<SmallVec<[oxc_syntax::node::NodeId; 4]>>> =
            if state.has_has_selector {
                let mut map = vec![SmallVec::new(); nodes.len()];
                for node in nodes {
                    let id = node.id();
                    let parent_id = nodes.parent_id(id);
                    if parent_id != id
                        && let Some(slot) = map.get_mut(parent_id.index())
                    {
                        slot.push(id);
                    }
                }
                Some(map)
            } else {
                None
            };
        let children_by_parent = children_map.as_deref();

        // Track which patterns have already fired on the current node so a `:matches(A, B)`,
        // comma-separated selector list, or mixed typed/unbounded selector within one pattern
        // only reports once per node.
        let mut reported_patterns: SmallVec<[usize; 8]> = SmallVec::new();
        for node in nodes {
            let ty = node.kind().ty();
            let typed_candidates = &state.by_type[ty as usize];
            if typed_candidates.is_empty() && state.unbounded_chains.is_empty() {
                continue;
            }

            reported_patterns.clear();
            for &(pattern_idx, chain_idx) in
                typed_candidates.iter().chain(state.unbounded_chains.iter())
            {
                if reported_patterns.contains(&pattern_idx) {
                    continue;
                }
                let pattern = &state.patterns[pattern_idx];
                if pattern.chains[chain_idx].matches_node(node, ctx, children_by_parent) {
                    reported_patterns.push(pattern_idx);
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
        let mut current = Some(root);
        for segment in &self.path {
            current = current.and_then(|val| val.get_prop(segment));
            if current.is_none() {
                break;
            }
        }
        match &self.matcher {
            AttrMatcher::Exists => current.is_some_and(|val| !matches!(val, PropValue::Null)),
            AttrMatcher::Equals(lit) => current.is_some_and(|val| val.equals_literal(lit)),
            AttrMatcher::NotEquals(lit) => current.is_none_or(|val| !val.equals_literal(lit)),
            AttrMatcher::Regex(re) => {
                current.and_then(PropValue::as_str).is_some_and(|s| re.is_match(s))
            }
            AttrMatcher::LessThan(n) => {
                current.and_then(PropValue::as_number).is_some_and(|v| v < *n)
            }
            AttrMatcher::LessEqual(n) => {
                current.and_then(PropValue::as_number).is_some_and(|v| v <= *n)
            }
            AttrMatcher::GreaterThan(n) => {
                current.and_then(PropValue::as_number).is_some_and(|v| v > *n)
            }
            AttrMatcher::GreaterEqual(n) => {
                current.and_then(PropValue::as_number).is_some_and(|v| v >= *n)
            }
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
    Statement(&'a Statement<'a>),
    Declaration(&'a Declaration<'a>),
    Expression(&'a Expression<'a>),
    Argument(&'a Argument<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    AssignmentTargetMaybeDefault(&'a AssignmentTargetMaybeDefault<'a>),
    BindingPattern(&'a BindingPattern<'a>),
    BindingProperty(&'a BindingProperty<'a>),
    PropertyKey(&'a PropertyKey<'a>),
    ObjectPropertyKind(&'a ObjectPropertyKind<'a>),
    ArrayExpressionElement(&'a ArrayExpressionElement<'a>),
    ClassElement(&'a ClassElement<'a>),
    ModuleExportName(&'a ModuleExportName<'a>),
    TSModuleReference(&'a TSModuleReference<'a>),
    ImportDeclarationSpecifier(&'a ImportDeclarationSpecifier<'a>),
    ExportDefaultDeclarationKind(&'a ExportDefaultDeclarationKind<'a>),
    ForStatementInit(&'a ForStatementInit<'a>),
    ForStatementLeft(&'a ForStatementLeft<'a>),
    JSXElementName(&'a JSXElementName<'a>),
    JSXMemberExpressionObject(&'a JSXMemberExpressionObject<'a>),
    JSXAttributeName(&'a JSXAttributeName<'a>),
    JSXAttributeValue(&'a JSXAttributeValue<'a>),
    JSXChild(&'a JSXChild<'a>),
    Arguments(&'a [Argument<'a>]),
    Expressions(&'a [Expression<'a>]),
    JSXAttributes(&'a [JSXAttributeItem<'a>]),
    JSXChildren(&'a [JSXChild<'a>]),
    Statements(&'a [Statement<'a>]),
    FormalParameters(&'a FormalParameters<'a>),
    VariableDeclarators(&'a [VariableDeclarator<'a>]),
    ObjectProperties(&'a [ObjectPropertyKind<'a>]),
    ArrayElements(&'a [ArrayExpressionElement<'a>]),
    BindingProperties(&'a [BindingProperty<'a>]),
    ArrayPatternElements(&'a [Option<BindingPattern<'a>>]),
    ClassElements(&'a [ClassElement<'a>]),
    ImportSpecifiers(&'a [ImportDeclarationSpecifier<'a>]),
    ExportSpecifiers(&'a [ExportSpecifier<'a>]),
    SwitchCases(&'a [SwitchCase<'a>]),
    TemplateElements(&'a [TemplateElement<'a>]),
    TSEnumMembers(&'a [TSEnumMember<'a>]),
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
            Self::Statement(stmt) => Some(statement_estree_type(stmt)),
            Self::Declaration(decl) => Some(declaration_estree_type(decl)),
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
            Self::AssignmentTargetMaybeDefault(target) => match target {
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(_) => {
                    Some("AssignmentPattern")
                }
                _ => target
                    .as_assignment_target()
                    .and_then(|t| Self::AssignmentTarget(t).estree_type()),
            },
            Self::BindingProperty(_) => Some("Property"),
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
            Self::ObjectPropertyKind(prop) => Some(match prop {
                ObjectPropertyKind::ObjectProperty(_) => "Property",
                ObjectPropertyKind::SpreadProperty(_) => "SpreadElement",
            }),
            Self::ArrayExpressionElement(el) => match el {
                ArrayExpressionElement::SpreadElement(_) => Some("SpreadElement"),
                ArrayExpressionElement::Elision(_) => None,
                _ => el.as_expression().map(expression_estree_type),
            },
            Self::ClassElement(el) => Some(match el {
                ClassElement::StaticBlock(_) => "StaticBlock",
                ClassElement::MethodDefinition(_) => "MethodDefinition",
                ClassElement::PropertyDefinition(_) => "PropertyDefinition",
                ClassElement::AccessorProperty(_) => "AccessorProperty",
                ClassElement::TSIndexSignature(_) => "TSIndexSignature",
            }),
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
            Self::ImportDeclarationSpecifier(spec) => Some(match spec {
                ImportDeclarationSpecifier::ImportSpecifier(_) => "ImportSpecifier",
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => "ImportDefaultSpecifier",
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    "ImportNamespaceSpecifier"
                }
            }),
            Self::ExportDefaultDeclarationKind(decl) => match decl {
                ExportDefaultDeclarationKind::FunctionDeclaration(_) => Some("FunctionDeclaration"),
                ExportDefaultDeclarationKind::ClassDeclaration(_) => Some("ClassDeclaration"),
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                    Some("TSInterfaceDeclaration")
                }
                _ => decl.as_expression().map(expression_estree_type),
            },
            Self::ForStatementInit(init) => match init {
                ForStatementInit::VariableDeclaration(_) => Some("VariableDeclaration"),
                _ => init.as_expression().map(expression_estree_type),
            },
            Self::ForStatementLeft(left) => match left {
                ForStatementLeft::VariableDeclaration(_) => Some("VariableDeclaration"),
                _ => left
                    .as_assignment_target()
                    .and_then(|t| Self::AssignmentTarget(t).estree_type()),
            },
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
            Self::JSXChild(child) => Some(match child {
                JSXChild::Text(_) => "JSXText",
                JSXChild::Element(_) => "JSXElement",
                JSXChild::Fragment(_) => "JSXFragment",
                JSXChild::ExpressionContainer(_) => "JSXExpressionContainer",
                JSXChild::Spread(_) => "JSXSpreadChild",
            }),
            Self::Str(_)
            | Self::Bool(_)
            | Self::Number(_)
            | Self::Null
            | Self::Arguments(_)
            | Self::Expressions(_)
            | Self::JSXAttributes(_)
            | Self::JSXChildren(_)
            | Self::Statements(_)
            | Self::FormalParameters(_)
            | Self::VariableDeclarators(_)
            | Self::ObjectProperties(_)
            | Self::ArrayElements(_)
            | Self::BindingProperties(_)
            | Self::ArrayPatternElements(_)
            | Self::ClassElements(_)
            | Self::ImportSpecifiers(_)
            | Self::ExportSpecifiers(_)
            | Self::SwitchCases(_)
            | Self::TemplateElements(_)
            | Self::TSEnumMembers(_) => None,
        }
    }

    fn get_prop(self, segment: &str) -> Option<PropValue<'a>> {
        if segment == "type" {
            return self.estree_type().map(PropValue::Str);
        }

        match self {
            Self::AstKind(kind) => get_ast_kind_prop(kind, segment),
            Self::Statement(stmt) => get_statement_prop(stmt, segment),
            Self::Declaration(decl) => get_declaration_prop(decl, segment),
            Self::Expression(expr) => get_expression_prop(expr, segment),
            Self::Argument(arg) => match arg {
                Argument::SpreadElement(spread) => match segment {
                    "argument" => Some(PropValue::Expression(&spread.argument)),
                    _ => None,
                },
                _ => get_expression_prop(arg.as_expression()?, segment),
            },
            Self::AssignmentTarget(target) => match target {
                AssignmentTarget::ArrayAssignmentTarget(arr) => {
                    get_ast_kind_prop(AstKind::ArrayAssignmentTarget(arr), segment)
                }
                AssignmentTarget::ObjectAssignmentTarget(obj) => {
                    get_ast_kind_prop(AstKind::ObjectAssignmentTarget(obj), segment)
                }
                _ => get_simple_assignment_target_prop(
                    target.as_simple_assignment_target()?,
                    segment,
                ),
            },
            Self::AssignmentTargetMaybeDefault(target) => match target {
                AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(def) => {
                    get_ast_kind_prop(AstKind::AssignmentTargetWithDefault(def), segment)
                }
                _ => Self::AssignmentTarget(target.as_assignment_target()?).get_prop(segment),
            },
            Self::BindingPattern(pat) => match pat {
                BindingPattern::BindingIdentifier(id) => match segment {
                    "name" => Some(PropValue::Str(id.name.as_str())),
                    _ => None,
                },
                BindingPattern::ObjectPattern(obj) => {
                    get_ast_kind_prop(AstKind::ObjectPattern(obj), segment)
                }
                BindingPattern::ArrayPattern(arr) => {
                    get_ast_kind_prop(AstKind::ArrayPattern(arr), segment)
                }
                BindingPattern::AssignmentPattern(assign) => match segment {
                    "left" => Some(PropValue::BindingPattern(&assign.left)),
                    "right" => Some(PropValue::Expression(&assign.right)),
                    _ => None,
                },
            },
            Self::BindingProperty(prop) => {
                get_ast_kind_prop(AstKind::BindingProperty(prop), segment)
            }
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
            Self::ObjectPropertyKind(prop) => match prop {
                ObjectPropertyKind::ObjectProperty(p) => {
                    get_ast_kind_prop(AstKind::ObjectProperty(p), segment)
                }
                ObjectPropertyKind::SpreadProperty(s) => {
                    get_ast_kind_prop(AstKind::SpreadElement(s), segment)
                }
            },
            Self::ArrayExpressionElement(el) => match el {
                ArrayExpressionElement::SpreadElement(s) => {
                    get_ast_kind_prop(AstKind::SpreadElement(s), segment)
                }
                ArrayExpressionElement::Elision(_) => None,
                _ => get_expression_prop(el.as_expression()?, segment),
            },
            Self::ClassElement(el) => match el {
                ClassElement::StaticBlock(b) => get_ast_kind_prop(AstKind::StaticBlock(b), segment),
                ClassElement::MethodDefinition(m) => {
                    get_ast_kind_prop(AstKind::MethodDefinition(m), segment)
                }
                ClassElement::PropertyDefinition(p) => {
                    get_ast_kind_prop(AstKind::PropertyDefinition(p), segment)
                }
                ClassElement::AccessorProperty(a) => {
                    get_ast_kind_prop(AstKind::AccessorProperty(a), segment)
                }
                ClassElement::TSIndexSignature(_) => None,
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
            Self::ImportDeclarationSpecifier(spec) => match spec {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    get_ast_kind_prop(AstKind::ImportSpecifier(s), segment)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    get_ast_kind_prop(AstKind::ImportDefaultSpecifier(s), segment)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    get_ast_kind_prop(AstKind::ImportNamespaceSpecifier(s), segment)
                }
            },
            Self::ExportDefaultDeclarationKind(decl) => match decl {
                ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                    get_ast_kind_prop(AstKind::Function(f), segment)
                }
                ExportDefaultDeclarationKind::ClassDeclaration(c) => {
                    get_ast_kind_prop(AstKind::Class(c), segment)
                }
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(i) => {
                    get_ast_kind_prop(AstKind::TSInterfaceDeclaration(i), segment)
                }
                _ => get_expression_prop(decl.as_expression()?, segment),
            },
            Self::ForStatementInit(init) => match init {
                ForStatementInit::VariableDeclaration(v) => {
                    get_ast_kind_prop(AstKind::VariableDeclaration(v), segment)
                }
                _ => get_expression_prop(init.as_expression()?, segment),
            },
            Self::ForStatementLeft(left) => match left {
                ForStatementLeft::VariableDeclaration(v) => {
                    get_ast_kind_prop(AstKind::VariableDeclaration(v), segment)
                }
                _ => Self::AssignmentTarget(left.as_assignment_target()?).get_prop(segment),
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
            Self::JSXChild(child) => match child {
                JSXChild::Text(t) => get_ast_kind_prop(AstKind::JSXText(t), segment),
                JSXChild::Element(el) => get_ast_kind_prop(AstKind::JSXElement(el), segment),
                JSXChild::Fragment(f) => get_ast_kind_prop(AstKind::JSXFragment(f), segment),
                JSXChild::ExpressionContainer(c) => {
                    get_ast_kind_prop(AstKind::JSXExpressionContainer(c), segment)
                }
                JSXChild::Spread(_) => None,
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
            Self::JSXChildren(children) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(children.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                children.get(idx).map(PropValue::JSXChild)
            }
            Self::Statements(stmts) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(stmts.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                stmts.get(idx).map(PropValue::Statement)
            }
            Self::FormalParameters(params) => {
                let total_len = params.items.len() + usize::from(params.rest.is_some());
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(total_len as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                if let Some(item) = params.items.get(idx) {
                    if item.has_modifier() || item.initializer.is_some() {
                        Some(PropValue::AstKind(AstKind::FormalParameter(item)))
                    } else {
                        Some(PropValue::BindingPattern(&item.pattern))
                    }
                } else if idx == params.items.len() {
                    params
                        .rest
                        .as_deref()
                        .map(|r| PropValue::AstKind(AstKind::FormalParameterRest(r)))
                } else {
                    None
                }
            }
            Self::VariableDeclarators(decls) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(decls.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                decls.get(idx).map(|d| PropValue::AstKind(AstKind::VariableDeclarator(d)))
            }
            Self::ObjectProperties(props) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(props.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                props.get(idx).map(PropValue::ObjectPropertyKind)
            }
            Self::ArrayElements(els) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(els.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                match els.get(idx)? {
                    ArrayExpressionElement::Elision(_) => Some(PropValue::Null),
                    other => Some(PropValue::ArrayExpressionElement(other)),
                }
            }
            Self::BindingProperties(props) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(props.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                props.get(idx).map(PropValue::BindingProperty)
            }
            Self::ArrayPatternElements(els) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(els.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                match els.get(idx)? {
                    Some(pat) => Some(PropValue::BindingPattern(pat)),
                    None => Some(PropValue::Null),
                }
            }
            Self::ClassElements(els) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(els.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                els.get(idx).map(PropValue::ClassElement)
            }
            Self::ImportSpecifiers(specs) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(specs.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                specs.get(idx).map(PropValue::ImportDeclarationSpecifier)
            }
            Self::ExportSpecifiers(specs) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(specs.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                specs.get(idx).map(|s| PropValue::AstKind(AstKind::ExportSpecifier(s)))
            }
            Self::SwitchCases(cases) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(cases.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                cases.get(idx).map(|c| PropValue::AstKind(AstKind::SwitchCase(c)))
            }
            Self::TemplateElements(quasis) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(quasis.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                quasis.get(idx).map(|q| PropValue::AstKind(AstKind::TemplateElement(q)))
            }
            Self::TSEnumMembers(members) => {
                if segment == "length" {
                    #[expect(clippy::cast_precision_loss)]
                    return Some(PropValue::Number(members.len() as f64));
                }
                let idx = segment.parse::<usize>().ok()?;
                members.get(idx).map(|m| PropValue::AstKind(AstKind::TSEnumMember(m)))
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

fn get_statement_prop<'a>(stmt: &'a Statement<'a>, segment: &str) -> Option<PropValue<'a>> {
    match stmt {
        Statement::BlockStatement(s) => get_ast_kind_prop(AstKind::BlockStatement(s), segment),
        Statement::BreakStatement(s) => get_ast_kind_prop(AstKind::BreakStatement(s), segment),
        Statement::ContinueStatement(s) => {
            get_ast_kind_prop(AstKind::ContinueStatement(s), segment)
        }
        Statement::DebuggerStatement(s) => {
            get_ast_kind_prop(AstKind::DebuggerStatement(s), segment)
        }
        Statement::DoWhileStatement(s) => get_ast_kind_prop(AstKind::DoWhileStatement(s), segment),
        Statement::EmptyStatement(s) => get_ast_kind_prop(AstKind::EmptyStatement(s), segment),
        Statement::ExpressionStatement(s) => {
            get_ast_kind_prop(AstKind::ExpressionStatement(s), segment)
        }
        Statement::ForInStatement(s) => get_ast_kind_prop(AstKind::ForInStatement(s), segment),
        Statement::ForOfStatement(s) => get_ast_kind_prop(AstKind::ForOfStatement(s), segment),
        Statement::ForStatement(s) => get_ast_kind_prop(AstKind::ForStatement(s), segment),
        Statement::IfStatement(s) => get_ast_kind_prop(AstKind::IfStatement(s), segment),
        Statement::LabeledStatement(s) => get_ast_kind_prop(AstKind::LabeledStatement(s), segment),
        Statement::ReturnStatement(s) => get_ast_kind_prop(AstKind::ReturnStatement(s), segment),
        Statement::SwitchStatement(s) => get_ast_kind_prop(AstKind::SwitchStatement(s), segment),
        Statement::ThrowStatement(s) => get_ast_kind_prop(AstKind::ThrowStatement(s), segment),
        Statement::TryStatement(s) => get_ast_kind_prop(AstKind::TryStatement(s), segment),
        Statement::WhileStatement(s) => get_ast_kind_prop(AstKind::WhileStatement(s), segment),
        Statement::WithStatement(s) => get_ast_kind_prop(AstKind::WithStatement(s), segment),
        Statement::VariableDeclaration(d) => {
            get_ast_kind_prop(AstKind::VariableDeclaration(d), segment)
        }
        Statement::FunctionDeclaration(d) => get_ast_kind_prop(AstKind::Function(d), segment),
        Statement::ClassDeclaration(d) => get_ast_kind_prop(AstKind::Class(d), segment),
        Statement::ImportDeclaration(d) => {
            get_ast_kind_prop(AstKind::ImportDeclaration(d), segment)
        }
        Statement::ExportAllDeclaration(d) => {
            get_ast_kind_prop(AstKind::ExportAllDeclaration(d), segment)
        }
        Statement::ExportDefaultDeclaration(d) => {
            get_ast_kind_prop(AstKind::ExportDefaultDeclaration(d), segment)
        }
        Statement::ExportDeclaration(d) => {
            get_ast_kind_prop(AstKind::ExportDeclaration(d), segment)
        }
        Statement::ExportNamedDeclaration(d) => {
            get_ast_kind_prop(AstKind::ExportNamedDeclaration(d), segment)
        }
        Statement::ExportFromDeclaration(d) => {
            get_ast_kind_prop(AstKind::ExportFromDeclaration(d), segment)
        }
        Statement::TSTypeAliasDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSTypeAliasDeclaration(d), segment)
        }
        Statement::TSInterfaceDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSInterfaceDeclaration(d), segment)
        }
        Statement::TSEnumDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSEnumDeclaration(d), segment)
        }
        Statement::TSExternalModuleDeclaration(_)
        | Statement::TSNamespaceDeclaration(_)
        | Statement::TSGlobalDeclaration(_)
        | Statement::TSNamespaceExportDeclaration(_) => None,
        Statement::TSImportEqualsDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSImportEqualsDeclaration(d), segment)
        }
        Statement::TSExportAssignment(d) => {
            get_ast_kind_prop(AstKind::TSExportAssignment(d), segment)
        }
    }
}

fn statement_estree_type(stmt: &Statement<'_>) -> &'static str {
    match stmt {
        Statement::BlockStatement(_) => "BlockStatement",
        Statement::BreakStatement(_) => "BreakStatement",
        Statement::ContinueStatement(_) => "ContinueStatement",
        Statement::DebuggerStatement(_) => "DebuggerStatement",
        Statement::DoWhileStatement(_) => "DoWhileStatement",
        Statement::EmptyStatement(_) => "EmptyStatement",
        Statement::ExpressionStatement(_) => "ExpressionStatement",
        Statement::ForInStatement(_) => "ForInStatement",
        Statement::ForOfStatement(_) => "ForOfStatement",
        Statement::ForStatement(_) => "ForStatement",
        Statement::IfStatement(_) => "IfStatement",
        Statement::LabeledStatement(_) => "LabeledStatement",
        Statement::ReturnStatement(_) => "ReturnStatement",
        Statement::SwitchStatement(_) => "SwitchStatement",
        Statement::ThrowStatement(_) => "ThrowStatement",
        Statement::TryStatement(_) => "TryStatement",
        Statement::WhileStatement(_) => "WhileStatement",
        Statement::WithStatement(_) => "WithStatement",
        Statement::VariableDeclaration(_) => "VariableDeclaration",
        Statement::FunctionDeclaration(_) => "FunctionDeclaration",
        Statement::ClassDeclaration(_) => "ClassDeclaration",
        Statement::ImportDeclaration(_) => "ImportDeclaration",
        Statement::ExportAllDeclaration(_) => "ExportAllDeclaration",
        Statement::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
        Statement::ExportDeclaration(_)
        | Statement::ExportNamedDeclaration(_)
        | Statement::ExportFromDeclaration(_) => "ExportNamedDeclaration",
        Statement::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
        Statement::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
        Statement::TSEnumDeclaration(_) => "TSEnumDeclaration",
        Statement::TSExternalModuleDeclaration(_)
        | Statement::TSNamespaceDeclaration(_)
        | Statement::TSGlobalDeclaration(_) => "TSModuleDeclaration",
        Statement::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
        Statement::TSExportAssignment(_) => "TSExportAssignment",
        Statement::TSNamespaceExportDeclaration(_) => "TSNamespaceExportDeclaration",
    }
}

fn get_declaration_prop<'a>(decl: &'a Declaration<'a>, segment: &str) -> Option<PropValue<'a>> {
    match decl {
        Declaration::VariableDeclaration(d) => {
            get_ast_kind_prop(AstKind::VariableDeclaration(d), segment)
        }
        Declaration::FunctionDeclaration(d) => get_ast_kind_prop(AstKind::Function(d), segment),
        Declaration::ClassDeclaration(d) => get_ast_kind_prop(AstKind::Class(d), segment),
        Declaration::TSTypeAliasDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSTypeAliasDeclaration(d), segment)
        }
        Declaration::TSInterfaceDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSInterfaceDeclaration(d), segment)
        }
        Declaration::TSEnumDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSEnumDeclaration(d), segment)
        }
        Declaration::TSExternalModuleDeclaration(_)
        | Declaration::TSNamespaceDeclaration(_)
        | Declaration::TSGlobalDeclaration(_) => None,
        Declaration::TSImportEqualsDeclaration(d) => {
            get_ast_kind_prop(AstKind::TSImportEqualsDeclaration(d), segment)
        }
    }
}

fn declaration_estree_type(decl: &Declaration<'_>) -> &'static str {
    match decl {
        Declaration::VariableDeclaration(_) => "VariableDeclaration",
        Declaration::FunctionDeclaration(_) => "FunctionDeclaration",
        Declaration::ClassDeclaration(_) => "ClassDeclaration",
        Declaration::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
        Declaration::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
        Declaration::TSEnumDeclaration(_) => "TSEnumDeclaration",
        Declaration::TSExternalModuleDeclaration(_)
        | Declaration::TSNamespaceDeclaration(_)
        | Declaration::TSGlobalDeclaration(_) => "TSModuleDeclaration",
        Declaration::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
    }
}

fn get_ast_kind_prop<'a>(kind: AstKind<'a>, segment: &str) -> Option<PropValue<'a>> {
    match kind {
        AstKind::Program(prog) => match segment {
            "body" => Some(PropValue::Statements(&prog.body)),
            "sourceType" => {
                Some(PropValue::Str(if prog.source_type.is_module() { "module" } else { "script" }))
            }
            _ => None,
        },
        AstKind::BlockStatement(block) => match segment {
            "body" => Some(PropValue::Statements(&block.body)),
            _ => None,
        },
        AstKind::FunctionBody(body) => match segment {
            "body" => Some(PropValue::Statements(&body.statements)),
            _ => None,
        },
        AstKind::StaticBlock(block) => match segment {
            "body" => Some(PropValue::Statements(&block.body)),
            _ => None,
        },
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
        AstKind::JSXText(text) => match segment {
            "value" => Some(PropValue::Str(text.value.as_str())),
            "raw" => text.raw.as_ref().map(|r| PropValue::Str(r.as_str())),
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
        AstKind::ChainExpression(chain) => match segment {
            "expression" => Some(match &chain.expression {
                ChainElement::CallExpression(call) => {
                    PropValue::AstKind(AstKind::CallExpression(call))
                }
                ChainElement::TSNonNullExpression(ts) => {
                    PropValue::AstKind(AstKind::TSNonNullExpression(ts))
                }
                ChainElement::ComputedMemberExpression(mem) => {
                    PropValue::AstKind(AstKind::ComputedMemberExpression(mem))
                }
                ChainElement::StaticMemberExpression(mem) => {
                    PropValue::AstKind(AstKind::StaticMemberExpression(mem))
                }
                ChainElement::PrivateFieldExpression(mem) => {
                    PropValue::AstKind(AstKind::PrivateFieldExpression(mem))
                }
            }),
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
            "specifiers" => Some(PropValue::ImportSpecifiers(
                decl.specifiers.as_ref().map_or(&[], |v| v.as_slice()),
            )),
            "importKind" => Some(PropValue::Str(import_or_export_kind_str(decl.import_kind))),
            _ => None,
        },
        AstKind::ImportSpecifier(spec) => match segment {
            "imported" => Some(PropValue::ModuleExportName(&spec.imported)),
            "local" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&spec.local))),
            "importKind" => Some(PropValue::Str(import_or_export_kind_str(spec.import_kind))),
            _ => None,
        },
        AstKind::ImportDefaultSpecifier(spec) => match segment {
            "local" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&spec.local))),
            _ => None,
        },
        AstKind::ImportNamespaceSpecifier(spec) => match segment {
            "local" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&spec.local))),
            _ => None,
        },
        AstKind::ExportDeclaration(decl) => match segment {
            "source" => Some(PropValue::Null),
            "specifiers" => Some(PropValue::ExportSpecifiers(&[])),
            "declaration" => Some(PropValue::Declaration(&decl.declaration)),
            "exportKind" => Some(PropValue::Str(import_or_export_kind_str(decl.export_kind()))),
            _ => None,
        },
        AstKind::ExportNamedDeclaration(decl) => match segment {
            "source" | "declaration" => Some(PropValue::Null),
            "specifiers" => Some(PropValue::ExportSpecifiers(&decl.specifiers)),
            "exportKind" => Some(PropValue::Str(import_or_export_kind_str(decl.export_kind))),
            _ => None,
        },
        AstKind::ExportFromDeclaration(decl) => match segment {
            "source" => Some(PropValue::AstKind(AstKind::StringLiteral(&decl.source))),
            "specifiers" => Some(PropValue::ExportSpecifiers(&decl.specifiers)),
            "declaration" => Some(PropValue::Null),
            "exportKind" => Some(PropValue::Str(import_or_export_kind_str(decl.export_kind))),
            _ => None,
        },
        AstKind::ExportDefaultDeclaration(decl) => match segment {
            "declaration" => Some(PropValue::ExportDefaultDeclarationKind(&decl.declaration)),
            "exportKind" => Some(PropValue::Str("value")),
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
        AstKind::ExportSpecifier(spec) => match segment {
            "local" => Some(PropValue::ModuleExportName(&spec.local)),
            "exported" => Some(PropValue::ModuleExportName(&spec.exported)),
            "exportKind" => Some(PropValue::Str(import_or_export_kind_str(spec.export_kind))),
            _ => None,
        },
        AstKind::ImportExpression(imp) => match segment {
            "source" => Some(PropValue::Expression(&imp.source)),
            "options" => imp
                .options
                .as_ref()
                .map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e))),
            _ => None,
        },
        AstKind::TSImportEqualsDeclaration(decl) => match segment {
            "id" => Some(PropValue::AstKind(AstKind::BindingIdentifier(&decl.id))),
            "moduleReference" => Some(PropValue::TSModuleReference(&decl.module_reference)),
            "importKind" => Some(PropValue::Str(import_or_export_kind_str(decl.import_kind))),
            "isExport" => Some(PropValue::Bool(false)),
            _ => None,
        },
        AstKind::TSExternalModuleReference(ext) => match segment {
            "expression" => Some(PropValue::AstKind(AstKind::StringLiteral(&ext.expression))),
            _ => None,
        },
        AstKind::TSExportAssignment(assign) => match segment {
            "expression" => Some(PropValue::Expression(&assign.expression)),
            _ => None,
        },
        AstKind::IfStatement(stmt) => match segment {
            "test" => Some(PropValue::Expression(&stmt.test)),
            "consequent" => Some(PropValue::Statement(&stmt.consequent)),
            "alternate" => stmt
                .alternate
                .as_ref()
                .map_or(Some(PropValue::Null), |s| Some(PropValue::Statement(s))),
            _ => None,
        },
        AstKind::WhileStatement(stmt) => match segment {
            "test" => Some(PropValue::Expression(&stmt.test)),
            "body" => Some(PropValue::Statement(&stmt.body)),
            _ => None,
        },
        AstKind::DoWhileStatement(stmt) => match segment {
            "test" => Some(PropValue::Expression(&stmt.test)),
            "body" => Some(PropValue::Statement(&stmt.body)),
            _ => None,
        },
        AstKind::ForStatement(stmt) => match segment {
            "init" => stmt
                .init
                .as_ref()
                .map_or(Some(PropValue::Null), |i| Some(PropValue::ForStatementInit(i))),
            "test" => {
                stmt.test.as_ref().map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e)))
            }
            "update" => stmt
                .update
                .as_ref()
                .map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e))),
            "body" => Some(PropValue::Statement(&stmt.body)),
            _ => None,
        },
        AstKind::ForInStatement(stmt) => match segment {
            "left" => Some(PropValue::ForStatementLeft(&stmt.left)),
            "right" => Some(PropValue::Expression(&stmt.right)),
            "body" => Some(PropValue::Statement(&stmt.body)),
            _ => None,
        },
        AstKind::ForOfStatement(stmt) => match segment {
            "left" => Some(PropValue::ForStatementLeft(&stmt.left)),
            "right" => Some(PropValue::Expression(&stmt.right)),
            "body" => Some(PropValue::Statement(&stmt.body)),
            "await" => Some(PropValue::Bool(stmt.r#await)),
            _ => None,
        },
        AstKind::SwitchStatement(stmt) => match segment {
            "discriminant" => Some(PropValue::Expression(&stmt.discriminant)),
            "cases" => Some(PropValue::SwitchCases(&stmt.cases)),
            _ => None,
        },
        AstKind::SwitchCase(case) => match segment {
            "test" => {
                case.test.as_ref().map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e)))
            }
            "consequent" => Some(PropValue::Statements(&case.consequent)),
            _ => None,
        },
        AstKind::TryStatement(stmt) => match segment {
            "block" => Some(PropValue::AstKind(AstKind::BlockStatement(&stmt.block))),
            "handler" => stmt.handler.as_deref().map_or(Some(PropValue::Null), |h| {
                Some(PropValue::AstKind(AstKind::CatchClause(h)))
            }),
            "finalizer" => stmt.finalizer.as_deref().map_or(Some(PropValue::Null), |f| {
                Some(PropValue::AstKind(AstKind::BlockStatement(f)))
            }),
            _ => None,
        },
        AstKind::CatchClause(clause) => match segment {
            "param" => clause
                .param
                .as_ref()
                .map_or(Some(PropValue::Null), |p| Some(PropValue::BindingPattern(&p.pattern))),
            "body" => Some(PropValue::AstKind(AstKind::BlockStatement(&clause.body))),
            _ => None,
        },
        AstKind::LabeledStatement(stmt) => match segment {
            "label" => Some(PropValue::AstKind(AstKind::LabelIdentifier(&stmt.label))),
            "body" => Some(PropValue::Statement(&stmt.body)),
            _ => None,
        },
        AstKind::BreakStatement(stmt) => match segment {
            "label" => stmt.label.as_ref().map_or(Some(PropValue::Null), |l| {
                Some(PropValue::AstKind(AstKind::LabelIdentifier(l)))
            }),
            _ => None,
        },
        AstKind::ContinueStatement(stmt) => match segment {
            "label" => stmt.label.as_ref().map_or(Some(PropValue::Null), |l| {
                Some(PropValue::AstKind(AstKind::LabelIdentifier(l)))
            }),
            _ => None,
        },
        AstKind::WithStatement(stmt) => match segment {
            "object" => Some(PropValue::Expression(&stmt.object)),
            "body" => Some(PropValue::Statement(&stmt.body)),
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
            "argument" => Some(PropValue::AssignmentTarget(u.argument.as_assignment_target())),
            _ => None,
        },
        AstKind::BinaryExpression(b) => match segment {
            "operator" => Some(PropValue::Str(b.operator.as_str())),
            "left" => Some(PropValue::Expression(&b.left)),
            "right" => Some(PropValue::Expression(&b.right)),
            _ => None,
        },
        AstKind::PrivateInExpression(b) => match segment {
            "operator" => Some(PropValue::Str("in")),
            "left" => Some(PropValue::AstKind(AstKind::PrivateIdentifier(&b.left))),
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
        AstKind::SequenceExpression(s) => match segment {
            "expressions" => Some(PropValue::Expressions(&s.expressions)),
            _ => None,
        },
        AstKind::VariableDeclarator(decl) => match segment {
            "id" => Some(PropValue::BindingPattern(&decl.id)),
            "init" => {
                decl.init.as_ref().map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e)))
            }
            "definite" => Some(PropValue::Bool(decl.definite)),
            _ => None,
        },
        AstKind::VariableDeclaration(decl) => match segment {
            "kind" => Some(PropValue::Str(decl.kind.as_str())),
            "declarations" => Some(PropValue::VariableDeclarators(&decl.declarations)),
            "declare" => Some(PropValue::Bool(decl.declare)),
            _ => None,
        },
        AstKind::AssignmentExpression(a) => match segment {
            "operator" => Some(PropValue::Str(a.operator.as_str())),
            "left" => Some(PropValue::AssignmentTarget(&a.left)),
            "right" => Some(PropValue::Expression(&a.right)),
            _ => None,
        },
        AstKind::ObjectExpression(o) => match segment {
            "properties" => Some(PropValue::ObjectProperties(&o.properties)),
            _ => None,
        },
        AstKind::ArrayExpression(a) => match segment {
            "elements" => Some(PropValue::ArrayElements(&a.elements)),
            _ => None,
        },
        AstKind::ObjectPattern(o) => match segment {
            "properties" => Some(PropValue::BindingProperties(&o.properties)),
            _ => None,
        },
        AstKind::ArrayPattern(a) => match segment {
            "elements" => Some(PropValue::ArrayPatternElements(&a.elements)),
            _ => None,
        },
        AstKind::AssignmentPattern(a) => match segment {
            "left" => Some(PropValue::BindingPattern(&a.left)),
            "right" => Some(PropValue::Expression(&a.right)),
            _ => None,
        },
        AstKind::AssignmentTargetWithDefault(a) => match segment {
            "left" => Some(PropValue::AssignmentTarget(&a.binding)),
            "right" => Some(PropValue::Expression(&a.init)),
            _ => None,
        },
        AstKind::BindingProperty(p) => match segment {
            "key" => Some(PropValue::PropertyKey(&p.key)),
            "value" => Some(PropValue::BindingPattern(&p.value)),
            "shorthand" => Some(PropValue::Bool(p.shorthand)),
            "computed" => Some(PropValue::Bool(p.computed)),
            "method" => Some(PropValue::Bool(false)),
            "kind" => Some(PropValue::Str("init")),
            _ => None,
        },
        AstKind::AssignmentTargetPropertyIdentifier(p) => match segment {
            "key" => Some(PropValue::AstKind(AstKind::IdentifierReference(&p.binding))),
            "value" => match &p.init {
                Some(init) => Some(PropValue::Expression(init)),
                None => Some(PropValue::AstKind(AstKind::IdentifierReference(&p.binding))),
            },
            "shorthand" => Some(PropValue::Bool(true)),
            "computed" | "method" => Some(PropValue::Bool(false)),
            "kind" => Some(PropValue::Str("init")),
            _ => None,
        },
        AstKind::AssignmentTargetPropertyProperty(p) => match segment {
            "key" => Some(PropValue::PropertyKey(&p.name)),
            "value" => Some(PropValue::AssignmentTargetMaybeDefault(&p.binding)),
            "shorthand" | "method" => Some(PropValue::Bool(false)),
            "computed" => Some(PropValue::Bool(p.computed)),
            "kind" => Some(PropValue::Str("init")),
            _ => None,
        },
        AstKind::BindingRestElement(r) => match segment {
            "argument" => Some(PropValue::BindingPattern(&r.argument)),
            _ => None,
        },
        AstKind::FormalParameterRest(r) => match segment {
            "argument" => Some(PropValue::BindingPattern(&r.rest.argument)),
            _ => None,
        },
        AstKind::AssignmentTargetRest(r) => match segment {
            "argument" => Some(PropValue::AssignmentTarget(&r.target)),
            _ => None,
        },
        AstKind::SpreadElement(s) => match segment {
            "argument" => Some(PropValue::Expression(&s.argument)),
            _ => None,
        },
        AstKind::FormalParameter(p) => {
            if p.has_modifier() {
                match segment {
                    "accessibility" => p
                        .accessibility
                        .map_or(Some(PropValue::Null), |a| Some(PropValue::Str(a.as_str()))),
                    "readonly" => Some(PropValue::Bool(p.readonly)),
                    "override" => Some(PropValue::Bool(p.r#override)),
                    "static" => Some(PropValue::Bool(false)),
                    "parameter" => Some(PropValue::BindingPattern(&p.pattern)),
                    _ => None,
                }
            } else if let Some(init) = &p.initializer {
                match segment {
                    "left" => Some(PropValue::BindingPattern(&p.pattern)),
                    "right" => Some(PropValue::Expression(init)),
                    "optional" => Some(PropValue::Bool(p.optional)),
                    _ => None,
                }
            } else {
                PropValue::BindingPattern(&p.pattern).get_prop(segment)
            }
        }
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
            "params" => Some(PropValue::FormalParameters(&f.params)),
            "body" => f.body.as_deref().map_or(Some(PropValue::Null), |b| {
                Some(PropValue::AstKind(AstKind::FunctionBody(b)))
            }),
            "async" => Some(PropValue::Bool(f.r#async)),
            "generator" => Some(PropValue::Bool(f.generator)),
            "expression" => Some(PropValue::Bool(false)),
            "declare" => Some(PropValue::Bool(f.declare)),
            _ => None,
        },
        AstKind::ArrowFunctionExpression(f) => match segment {
            "id" => Some(PropValue::Null),
            "params" => Some(PropValue::FormalParameters(&f.params)),
            "body" => match &f.body {
                ArrowFunctionBody::FunctionBody(b) => {
                    Some(PropValue::AstKind(AstKind::FunctionBody(b)))
                }
                _ => f.body.as_expression().map(PropValue::Expression),
            },
            "async" => Some(PropValue::Bool(f.r#async)),
            "generator" => Some(PropValue::Bool(false)),
            "expression" => Some(PropValue::Bool(f.is_expression())),
            _ => None,
        },
        AstKind::Class(c) => match segment {
            "id" => c.id.as_ref().map_or(Some(PropValue::Null), |id| {
                Some(PropValue::AstKind(AstKind::BindingIdentifier(id)))
            }),
            "superClass" => c
                .heritage_expression()
                .map_or(Some(PropValue::Null), |s| Some(PropValue::Expression(s))),
            "body" => Some(PropValue::AstKind(AstKind::ClassBody(&c.body))),
            "abstract" => Some(PropValue::Bool(c.r#abstract)),
            "declare" => Some(PropValue::Bool(c.declare)),
            _ => None,
        },
        AstKind::ClassBody(body) => match segment {
            "body" => Some(PropValue::ClassElements(&body.body)),
            _ => None,
        },
        AstKind::MethodDefinition(m) => match segment {
            "key" => Some(PropValue::PropertyKey(&m.key)),
            "value" => Some(PropValue::AstKind(AstKind::Function(&m.value))),
            "static" => Some(PropValue::Bool(m.r#static)),
            "computed" => Some(PropValue::Bool(m.computed)),
            "override" => Some(PropValue::Bool(m.r#override)),
            "optional" => Some(PropValue::Bool(m.optional)),
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
            "readonly" => Some(PropValue::Bool(p.readonly)),
            "override" => Some(PropValue::Bool(p.r#override)),
            "optional" => Some(PropValue::Bool(p.optional)),
            "declare" => Some(PropValue::Bool(p.declare)),
            "definite" => Some(PropValue::Bool(p.definite)),
            _ => None,
        },
        AstKind::AccessorProperty(p) => match segment {
            "key" => Some(PropValue::PropertyKey(&p.key)),
            "value" => {
                p.value.as_ref().map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e)))
            }
            "static" => Some(PropValue::Bool(p.r#static)),
            "computed" => Some(PropValue::Bool(p.computed)),
            "definite" => Some(PropValue::Bool(p.definite)),
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
            "quasis" => Some(PropValue::TemplateElements(&t.quasis)),
            _ => None,
        },
        AstKind::TemplateElement(el) => match segment {
            "tail" => Some(PropValue::Bool(el.tail)),
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
            "directive" => Some(PropValue::Null),
            _ => None,
        },
        AstKind::Directive(d) => match segment {
            "expression" => Some(PropValue::AstKind(AstKind::StringLiteral(&d.expression))),
            "directive" => Some(PropValue::Str(d.directive.as_str())),
            _ => None,
        },
        AstKind::JSXElement(el) => match segment {
            "openingElement" => {
                Some(PropValue::AstKind(AstKind::JSXOpeningElement(&el.opening_element)))
            }
            "closingElement" => el.closing_element.as_deref().map_or(Some(PropValue::Null), |c| {
                Some(PropValue::AstKind(AstKind::JSXClosingElement(c)))
            }),
            "children" => Some(PropValue::JSXChildren(&el.children)),
            _ => None,
        },
        AstKind::JSXOpeningElement(el) => match segment {
            "name" => Some(PropValue::JSXElementName(&el.name)),
            "attributes" => Some(PropValue::JSXAttributes(&el.attributes)),
            _ => None,
        },
        AstKind::JSXClosingElement(el) => match segment {
            "name" => Some(PropValue::JSXElementName(&el.name)),
            _ => None,
        },
        AstKind::JSXFragment(f) => match segment {
            "children" => Some(PropValue::JSXChildren(&f.children)),
            _ => None,
        },
        AstKind::JSXExpressionContainer(c) => match segment {
            "expression" => c.expression.as_expression().map(PropValue::Expression),
            _ => None,
        },
        AstKind::JSXSpreadAttribute(s) => match segment {
            "argument" => Some(PropValue::Expression(&s.argument)),
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
            "members" => Some(PropValue::TSEnumMembers(&decl.body.members)),
            "const" => Some(PropValue::Bool(decl.r#const)),
            "declare" => Some(PropValue::Bool(decl.declare)),
            _ => None,
        },
        AstKind::TSEnumMember(member) => match segment {
            "initializer" => member
                .initializer
                .as_ref()
                .map_or(Some(PropValue::Null), |e| Some(PropValue::Expression(e))),
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
        AstKind::TSTypeReference(r) => match segment {
            "typeName" => Some(match &r.type_name {
                TSTypeName::IdentifierReference(id) => {
                    PropValue::AstKind(AstKind::IdentifierReference(id))
                }
                TSTypeName::QualifiedName(q) => PropValue::AstKind(AstKind::TSQualifiedName(q)),
                TSTypeName::ThisExpression(t) => PropValue::AstKind(AstKind::ThisExpression(t)),
            }),
            _ => None,
        },
        AstKind::TSQualifiedName(q) => match segment {
            "left" => Some(match &q.left {
                TSTypeName::IdentifierReference(id) => {
                    PropValue::AstKind(AstKind::IdentifierReference(id))
                }
                TSTypeName::QualifiedName(inner) => {
                    PropValue::AstKind(AstKind::TSQualifiedName(inner))
                }
                TSTypeName::ThisExpression(t) => PropValue::AstKind(AstKind::ThisExpression(t)),
            }),
            "right" => Some(PropValue::AstKind(AstKind::IdentifierName(&q.right))),
            _ => None,
        },
        AstKind::TSImportType(imp) => match segment {
            "source" => Some(PropValue::AstKind(AstKind::StringLiteral(&imp.source))),
            "qualifier" => Some(match &imp.qualifier {
                Some(TSImportTypeQualifier::Identifier(id)) => {
                    PropValue::AstKind(AstKind::IdentifierName(id))
                }
                Some(TSImportTypeQualifier::QualifiedName(q)) => {
                    PropValue::AstKind(AstKind::TSImportTypeQualifiedName(q))
                }
                None => PropValue::Null,
            }),
            _ => None,
        },
        AstKind::TSImportTypeQualifiedName(q) => match segment {
            "left" => Some(match &q.left {
                TSImportTypeQualifier::Identifier(id) => {
                    PropValue::AstKind(AstKind::IdentifierName(id))
                }
                TSImportTypeQualifier::QualifiedName(inner) => {
                    PropValue::AstKind(AstKind::TSImportTypeQualifiedName(inner))
                }
            }),
            "right" => Some(PropValue::AstKind(AstKind::IdentifierName(&q.right))),
            _ => None,
        },
        AstKind::Decorator(d) => match segment {
            "expression" => Some(PropValue::Expression(&d.expression)),
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
        | AstKind::LabelIdentifier(_)
        | AstKind::TSIndexSignatureName(_) => "Identifier",
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
        AstKind::BlockStatement(_) | AstKind::FunctionBody(_) => "BlockStatement",
        AstKind::VariableDeclaration(_) => "VariableDeclaration",
        AstKind::VariableDeclarator(_) => "VariableDeclarator",
        AstKind::EmptyStatement(_) => "EmptyStatement",
        AstKind::ExpressionStatement(_) | AstKind::Directive(_) => "ExpressionStatement",
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
        AstKind::FormalParameter(param) => {
            if param.has_modifier() {
                "TSParameterProperty"
            } else if param.initializer.is_some() {
                "AssignmentPattern"
            } else {
                match &param.pattern {
                    BindingPattern::BindingIdentifier(_) => "Identifier",
                    BindingPattern::ObjectPattern(_) => "ObjectPattern",
                    BindingPattern::ArrayPattern(_) => "ArrayPattern",
                    BindingPattern::AssignmentPattern(_) => "AssignmentPattern",
                }
            }
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
        AstKind::ExportDeclaration(_)
        | AstKind::ExportNamedDeclaration(_)
        | AstKind::ExportFromDeclaration(_) => "ExportNamedDeclaration",
        AstKind::ExportDefaultDeclaration(_) => "ExportDefaultDeclaration",
        AstKind::ExportAllDeclaration(_) => "ExportAllDeclaration",
        AstKind::ExportSpecifier(_) => "ExportSpecifier",
        AstKind::ImportAttribute(_) => "ImportAttribute",
        AstKind::V8IntrinsicExpression(_) => "V8IntrinsicExpression",
        AstKind::Hashbang(_) => "Hashbang",
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
        AstKind::JSXOpeningFragment(_) => "JSXOpeningFragment",
        AstKind::JSXClosingFragment(_) => "JSXClosingFragment",
        AstKind::JSXNamespacedName(_) => "JSXNamespacedName",
        AstKind::JSXMemberExpression(_) => "JSXMemberExpression",
        AstKind::JSXExpressionContainer(_) => "JSXExpressionContainer",
        AstKind::JSXEmptyExpression(_) => "JSXEmptyExpression",
        AstKind::JSXAttribute(_) => "JSXAttribute",
        AstKind::JSXSpreadAttribute(_) => "JSXSpreadAttribute",
        AstKind::JSXIdentifier(_) => "JSXIdentifier",
        AstKind::JSXSpreadChild(_) => "JSXSpreadChild",
        AstKind::JSXText(_) => "JSXText",
        AstKind::TSThisParameter(_) => "TSThisParameter",
        AstKind::TSEnumDeclaration(_) => "TSEnumDeclaration",
        AstKind::TSEnumBody(_) => "TSEnumBody",
        AstKind::TSEnumMember(_) => "TSEnumMember",
        AstKind::TSTypeAnnotation(_) => "TSTypeAnnotation",
        AstKind::TSLiteralType(_) => "TSLiteralType",
        AstKind::TSConditionalType(_) => "TSConditionalType",
        AstKind::TSUnionType(_) => "TSUnionType",
        AstKind::TSIntersectionType(_) => "TSIntersectionType",
        AstKind::TSParenthesizedType(_) => "TSParenthesizedType",
        AstKind::TSTypeOperator(_) => "TSTypeOperator",
        AstKind::TSArrayType(_) => "TSArrayType",
        AstKind::TSIndexedAccessType(_) => "TSIndexedAccessType",
        AstKind::TSTupleType(_) => "TSTupleType",
        AstKind::TSNamedTupleMember(_) => "TSNamedTupleMember",
        AstKind::TSOptionalType(_) => "TSOptionalType",
        AstKind::TSRestType(_) => "TSRestType",
        AstKind::TSAnyKeyword(_) => "TSAnyKeyword",
        AstKind::TSStringKeyword(_) => "TSStringKeyword",
        AstKind::TSBooleanKeyword(_) => "TSBooleanKeyword",
        AstKind::TSNumberKeyword(_) => "TSNumberKeyword",
        AstKind::TSNeverKeyword(_) => "TSNeverKeyword",
        AstKind::TSIntrinsicKeyword(_) => "TSIntrinsicKeyword",
        AstKind::TSUnknownKeyword(_) => "TSUnknownKeyword",
        AstKind::TSNullKeyword(_) => "TSNullKeyword",
        AstKind::TSUndefinedKeyword(_) => "TSUndefinedKeyword",
        AstKind::TSVoidKeyword(_) => "TSVoidKeyword",
        AstKind::TSSymbolKeyword(_) => "TSSymbolKeyword",
        AstKind::TSThisType(_) => "TSThisType",
        AstKind::TSObjectKeyword(_) => "TSObjectKeyword",
        AstKind::TSBigIntKeyword(_) => "TSBigIntKeyword",
        AstKind::TSTypeReference(_) => "TSTypeReference",
        AstKind::TSQualifiedName(_) | AstKind::TSImportTypeQualifiedName(_) => "TSQualifiedName",
        AstKind::TSTypeParameterInstantiation(_) => "TSTypeParameterInstantiation",
        AstKind::TSTypeParameter(_) => "TSTypeParameter",
        AstKind::TSTypeParameterDeclaration(_) => "TSTypeParameterDeclaration",
        AstKind::TSTypeAliasDeclaration(_) => "TSTypeAliasDeclaration",
        AstKind::TSClassImplements(_) => "TSClassImplements",
        AstKind::TSInterfaceDeclaration(_) => "TSInterfaceDeclaration",
        AstKind::TSInterfaceBody(_) => "TSInterfaceBody",
        AstKind::TSPropertySignature(_) => "TSPropertySignature",
        AstKind::TSIndexSignature(_) => "TSIndexSignature",
        AstKind::TSCallSignatureDeclaration(_) => "TSCallSignatureDeclaration",
        AstKind::TSMethodSignature(_) => "TSMethodSignature",
        AstKind::TSConstructSignatureDeclaration(_) => "TSConstructSignatureDeclaration",
        AstKind::TSInterfaceHeritage(_) => "TSInterfaceHeritage",
        AstKind::TSTypePredicate(_) => "TSTypePredicate",
        AstKind::TSExternalModuleDeclaration(_)
        | AstKind::TSNamespaceDeclaration(_)
        | AstKind::TSGlobalDeclaration(_) => "TSModuleDeclaration",
        AstKind::TSModuleBlock(_) => "TSModuleBlock",
        AstKind::TSTypeLiteral(_) => "TSTypeLiteral",
        AstKind::TSInferType(_) => "TSInferType",
        AstKind::TSTypeQuery(_) => "TSTypeQuery",
        AstKind::TSImportType(_) => "TSImportType",
        AstKind::TSFunctionType(_) => "TSFunctionType",
        AstKind::TSConstructorType(_) => "TSConstructorType",
        AstKind::TSMappedType(_) => "TSMappedType",
        AstKind::TSTemplateLiteralType(_) => "TSTemplateLiteralType",
        AstKind::TSAsExpression(_) => "TSAsExpression",
        AstKind::TSSatisfiesExpression(_) => "TSSatisfiesExpression",
        AstKind::TSTypeAssertion(_) => "TSTypeAssertion",
        AstKind::TSImportEqualsDeclaration(_) => "TSImportEqualsDeclaration",
        AstKind::TSExternalModuleReference(_) => "TSExternalModuleReference",
        AstKind::TSNonNullExpression(_) => "TSNonNullExpression",
        AstKind::Decorator(_) => "Decorator",
        AstKind::TSExportAssignment(_) => "TSExportAssignment",
        AstKind::TSNamespaceExportDeclaration(_) => "TSNamespaceExportDeclaration",
        AstKind::TSInstantiationExpression(_) => "TSInstantiationExpression",
        AstKind::JSDocNullableType(_) => "JSDocNullableType",
        AstKind::JSDocNonNullableType(_) => "JSDocNonNullableType",
        AstKind::JSDocUnknownType(_) => "JSDocUnknownType",
        AstKind::Elision(_)
        | AstKind::ParenthesizedExpression(_)
        | AstKind::CatchParameter(_)
        | AstKind::FormalParameters(_)
        | AstKind::WithClause(_) => "",
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
            AstType::TSIndexSignatureName,
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
        "MetaProperty" => &[AstType::ImportMeta, AstType::NewTarget],
        "ImportDeclaration" => &[AstType::ImportDeclaration],
        "ImportSpecifier" => &[AstType::ImportSpecifier],
        "ImportDefaultSpecifier" => &[AstType::ImportDefaultSpecifier],
        "ImportNamespaceSpecifier" => &[AstType::ImportNamespaceSpecifier],
        "ImportAttribute" => &[AstType::ImportAttribute],
        "ExportNamedDeclaration" => &[
            AstType::ExportDeclaration,
            AstType::ExportNamedDeclaration,
            AstType::ExportFromDeclaration,
        ],
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
        "AccessorProperty" => &[AstType::AccessorProperty],
        "StaticBlock" => &[AstType::StaticBlock],
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
        "AssignmentPattern" => &[
            AstType::AssignmentPattern,
            AstType::AssignmentTargetWithDefault,
            AstType::FormalParameter,
        ],
        "TSParameterProperty" => &[AstType::FormalParameter],
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
        "BlockStatement" => &[AstType::BlockStatement, AstType::FunctionBody],
        "ExpressionStatement" => &[AstType::ExpressionStatement, AstType::Directive],
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
        "JSXOpeningFragment" => &[AstType::JSXOpeningFragment],
        "JSXClosingFragment" => &[AstType::JSXClosingFragment],
        "JSXAttribute" => &[AstType::JSXAttribute],
        "JSXSpreadAttribute" => &[AstType::JSXSpreadAttribute],
        "JSXIdentifier" => &[AstType::JSXIdentifier],
        "JSXMemberExpression" => &[AstType::JSXMemberExpression],
        "JSXNamespacedName" => &[AstType::JSXNamespacedName],
        "JSXExpressionContainer" => &[AstType::JSXExpressionContainer],
        "JSXEmptyExpression" => &[AstType::JSXEmptyExpression],
        "JSXSpreadChild" => &[AstType::JSXSpreadChild],
        "JSXText" => &[AstType::JSXText],
        "TSThisParameter" => &[AstType::TSThisParameter],
        "TSEnumDeclaration" => &[AstType::TSEnumDeclaration],
        "TSEnumBody" => &[AstType::TSEnumBody],
        "TSEnumMember" => &[AstType::TSEnumMember],
        "TSTypeAnnotation" => &[AstType::TSTypeAnnotation],
        "TSLiteralType" => &[AstType::TSLiteralType],
        "TSConditionalType" => &[AstType::TSConditionalType],
        "TSUnionType" => &[AstType::TSUnionType],
        "TSIntersectionType" => &[AstType::TSIntersectionType],
        "TSParenthesizedType" => &[AstType::TSParenthesizedType],
        "TSTypeOperator" => &[AstType::TSTypeOperator],
        "TSArrayType" => &[AstType::TSArrayType],
        "TSIndexedAccessType" => &[AstType::TSIndexedAccessType],
        "TSTupleType" => &[AstType::TSTupleType],
        "TSNamedTupleMember" => &[AstType::TSNamedTupleMember],
        "TSOptionalType" => &[AstType::TSOptionalType],
        "TSRestType" => &[AstType::TSRestType],
        "TSAnyKeyword" => &[AstType::TSAnyKeyword],
        "TSStringKeyword" => &[AstType::TSStringKeyword],
        "TSBooleanKeyword" => &[AstType::TSBooleanKeyword],
        "TSNumberKeyword" => &[AstType::TSNumberKeyword],
        "TSNeverKeyword" => &[AstType::TSNeverKeyword],
        "TSIntrinsicKeyword" => &[AstType::TSIntrinsicKeyword],
        "TSUnknownKeyword" => &[AstType::TSUnknownKeyword],
        "TSNullKeyword" => &[AstType::TSNullKeyword],
        "TSUndefinedKeyword" => &[AstType::TSUndefinedKeyword],
        "TSVoidKeyword" => &[AstType::TSVoidKeyword],
        "TSSymbolKeyword" => &[AstType::TSSymbolKeyword],
        "TSThisType" => &[AstType::TSThisType],
        "TSObjectKeyword" => &[AstType::TSObjectKeyword],
        "TSBigIntKeyword" => &[AstType::TSBigIntKeyword],
        "TSTypeReference" => &[AstType::TSTypeReference],
        "TSQualifiedName" => &[AstType::TSQualifiedName, AstType::TSImportTypeQualifiedName],
        "TSTypeParameterInstantiation" => &[AstType::TSTypeParameterInstantiation],
        "TSTypeParameter" => &[AstType::TSTypeParameter],
        "TSTypeParameterDeclaration" => &[AstType::TSTypeParameterDeclaration],
        "TSTypeAliasDeclaration" => &[AstType::TSTypeAliasDeclaration],
        "TSClassImplements" => &[AstType::TSClassImplements],
        "TSInterfaceDeclaration" => &[AstType::TSInterfaceDeclaration],
        "TSInterfaceBody" => &[AstType::TSInterfaceBody],
        "TSPropertySignature" => &[AstType::TSPropertySignature],
        "TSIndexSignature" => &[AstType::TSIndexSignature],
        "TSCallSignatureDeclaration" => &[AstType::TSCallSignatureDeclaration],
        "TSMethodSignature" => &[AstType::TSMethodSignature],
        "TSConstructSignatureDeclaration" => &[AstType::TSConstructSignatureDeclaration],
        "TSInterfaceHeritage" => &[AstType::TSInterfaceHeritage],
        "TSTypePredicate" => &[AstType::TSTypePredicate],
        "TSModuleDeclaration" => &[
            AstType::TSExternalModuleDeclaration,
            AstType::TSNamespaceDeclaration,
            AstType::TSGlobalDeclaration,
        ],
        "TSModuleBlock" => &[AstType::TSModuleBlock],
        "TSTypeLiteral" => &[AstType::TSTypeLiteral],
        "TSInferType" => &[AstType::TSInferType],
        "TSTypeQuery" => &[AstType::TSTypeQuery],
        "TSImportType" => &[AstType::TSImportType],
        "TSFunctionType" => &[AstType::TSFunctionType],
        "TSConstructorType" => &[AstType::TSConstructorType],
        "TSMappedType" => &[AstType::TSMappedType],
        "TSTemplateLiteralType" => &[AstType::TSTemplateLiteralType],
        "TSAsExpression" => &[AstType::TSAsExpression],
        "TSSatisfiesExpression" => &[AstType::TSSatisfiesExpression],
        "TSTypeAssertion" => &[AstType::TSTypeAssertion],
        "TSNonNullExpression" => &[AstType::TSNonNullExpression],
        "Decorator" => &[AstType::Decorator],
        "TSExportAssignment" => &[AstType::TSExportAssignment],
        "TSNamespaceExportDeclaration" => &[AstType::TSNamespaceExportDeclaration],
        "TSInstantiationExpression" => &[AstType::TSInstantiationExpression],
        "V8IntrinsicExpression" => &[AstType::V8IntrinsicExpression],
        "Hashbang" => &[AstType::Hashbang],
        "JSDocNullableType" => &[AstType::JSDocNullableType],
        "JSDocNonNullableType" => &[AstType::JSDocNonNullableType],
        "JSDocUnknownType" => &[AstType::JSDocUnknownType],
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

fn is_supported_attr_segment(segment: &str) -> bool {
    if segment.parse::<usize>().is_ok() {
        return true;
    }
    matches!(
        segment,
        "type"
            | "name"
            | "value"
            | "raw"
            | "bigint"
            | "pattern"
            | "length"
            | "callee"
            | "arguments"
            | "optional"
            | "object"
            | "property"
            | "computed"
            | "source"
            | "specifiers"
            | "importKind"
            | "exportKind"
            | "exported"
            | "local"
            | "imported"
            | "declaration"
            | "declarations"
            | "options"
            | "id"
            | "moduleReference"
            | "isExport"
            | "expression"
            | "expressions"
            | "test"
            | "consequent"
            | "alternate"
            | "init"
            | "update"
            | "body"
            | "left"
            | "right"
            | "await"
            | "discriminant"
            | "cases"
            | "block"
            | "handler"
            | "finalizer"
            | "param"
            | "params"
            | "label"
            | "operator"
            | "prefix"
            | "argument"
            | "definite"
            | "kind"
            | "declare"
            | "properties"
            | "elements"
            | "key"
            | "shorthand"
            | "method"
            | "accessibility"
            | "readonly"
            | "override"
            | "static"
            | "parameter"
            | "async"
            | "generator"
            | "superClass"
            | "tag"
            | "quasi"
            | "quasis"
            | "tail"
            | "delegate"
            | "directive"
            | "openingElement"
            | "closingElement"
            | "children"
            | "attributes"
            | "selfClosing"
            | "namespace"
            | "members"
            | "const"
            | "initializer"
            | "sourceType"
            | "typeName"
            | "qualifier"
    )
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
        if path.is_empty() || !path.iter().all(|seg| is_supported_attr_segment(seg)) {
            return None;
        }
        return Some(AttrPredicate { path, matcher: AttrMatcher::Exists });
    };

    let lhs = inner[..op_pos].trim();
    let op = &inner[op_pos..op_pos + op_len];
    let rhs = inner[op_pos + op_len..].trim();

    let path: Vec<String> =
        lhs.split('.').map(str::trim).filter(|s| !s.is_empty()).map(ToString::to_string).collect();
    if path.is_empty() || rhs.is_empty() || !path.iter().all(|seg| is_supported_attr_segment(seg)) {
        return None;
    }

    let matcher = match op {
        "=" => {
            if rhs.starts_with('/') {
                AttrMatcher::Regex(parse_regex_literal(rhs)?)
            } else {
                AttrMatcher::Equals(parse_attr_literal(rhs)?)
            }
        }
        "!=" => {
            if rhs.starts_with('/') {
                return None;
            }
            AttrMatcher::NotEquals(parse_attr_literal(rhs)?)
        }
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
    let mut case_insensitive = false;
    let mut multi_line = false;
    let mut dot_matches_new_line = false;
    let mut unicode = false;
    for flag in flags.chars() {
        match flag {
            'i' if !case_insensitive => case_insensitive = true,
            'm' if !multi_line => multi_line = true,
            's' if !dot_matches_new_line => dot_matches_new_line = true,
            'u' if !unicode => unicode = true,
            _ => return None,
        }
    }
    RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .multi_line(multi_line)
        .dot_matches_new_line(dot_matches_new_line)
        .build()
        .ok()
}

fn parse_attr_literal(rhs: &str) -> Option<AttrLiteral> {
    if rhs.starts_with('\'') || rhs.starts_with('"') {
        let quote = rhs.chars().next()?;
        if rhs.len() < 2 || !rhs.ends_with(quote) {
            return None;
        }
        let inner = &rhs[1..rhs.len() - 1];
        let mut unescaped = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                unescaped.push(chars.next()?);
            } else {
                unescaped.push(ch);
            }
        }
        return Some(AttrLiteral::Str(unescaped));
    }
    match rhs {
        "true" => Some(AttrLiteral::Bool(true)),
        "false" => Some(AttrLiteral::Bool(false)),
        "null" => Some(AttrLiteral::Null),
        _ => {
            if let Ok(n) = rhs.parse::<f64>() {
                Some(AttrLiteral::Number(n))
            } else if rhs.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '$'))
            {
                Some(AttrLiteral::Str(rhs.to_string()))
            } else {
                None
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

    let bad_regex = NoRestrictedSyntax::from_configuration(json!(["Identifier[name=/(/]"]));
    assert!(bad_regex.is_err(), "malformed regex literal in selector must fail closed");

    let bad_flags = NoRestrictedSyntax::from_configuration(json!(["Identifier[name=/foo/xyz]"]));
    assert!(bad_flags.is_err(), "unsupported regex flags in selector must fail closed");

    let bad_prop =
        NoRestrictedSyntax::from_configuration(json!(["FunctionDeclaration[unknownProp=1]"]));
    assert!(bad_prop.is_err(), "unsupported ESTree attribute segment must fail closed");
}

#[test]
fn test_estree_normalization_and_deduplication() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (r"foo?.();", Some(json!(["ExpressionStatement > CallExpression"]))),
        (r"eval('1');", Some(json!(["CallExpression[callee.name!='eval']"]))),
        (r"function f() {}", Some(json!(["FunctionDeclaration[params.length>0]"]))),
    ];

    let fail = vec![
        (r"foo?.();", Some(json!(["ChainExpression > CallExpression"]))),
        (r"obj.run();", Some(json!(["CallExpression[callee.name!='eval']"]))),
        (r"function f(x) {}", Some(json!(["FunctionDeclaration > Identifier[name='x']"]))),
        (
            r"function f(x = 1) {}",
            Some(json!(["FunctionDeclaration > AssignmentPattern > Identifier[name='x']"])),
        ),
        (r"function f(x) {}", Some(json!(["FunctionDeclaration[params.length>0]"]))),
        (r"function f() { return 1; }", Some(json!(["FunctionDeclaration > BlockStatement"]))),
        (r"foo(eval('x'));", Some(json!(["CallExpression:has(Identifier[name='eval'])"]))),
        (r"type T = string | number;", Some(json!(["*[type='TSUnionType']"]))),
        (r"type T = Foo.Bar;", Some(json!(["TSQualifiedName[left.name='Foo']"]))),
        (
            r#"type T = import("./module").a.b;"#,
            Some(json!(["TSQualifiedName[left.name='a'][right.name='b']"])),
        ),
    ];

    Tester::new(NoRestrictedSyntax::NAME, NoRestrictedSyntax::PLUGIN, pass, fail).test();
}

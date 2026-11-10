//! Inline immediately invoked functions whose body is a single expression.
//!
//! `(function () { return x })()` → `x`
//! `(function (a, b) { return a * b })(2, 3)` → `2 * 3`
//! `((a) => a + 1)(1, f())` → `(f(), 1 + 1)`
//!
//! Terser performs this as part of `inline`/`reduce_funcs`. The callee must
//! not observe its own activation: no `this`, `arguments`, `new.target`,
//! `super`, direct `eval`, own-name reference, or class body (whose `this`
//! scoping is not tracked here). Parameters are replaced only by literal
//! arguments, so moving the read from the call site into the body cannot
//! change the value it observes.

use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, VisitMut, walk, walk_mut};
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::{MayHaveSideEffects, MayHaveSideEffectsContext},
};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolId};

use crate::{TraverseCtx, generated::ancestor::Ancestor};

use super::PeepholeOptimizations;

/// The longest string literal duplicated into more than one read.
const MAX_DUPLICATED_STRING_LEN: usize = 8;

impl<'a> PeepholeOptimizations {
    pub fn inline_function_iife(e: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        let Expression::CallExpression(call) = e else { return };
        if call.optional || call.arguments.iter().any(Argument::is_spread) {
            return;
        }
        let (params, function_like) = match &call.callee {
            Expression::FunctionExpression(f) => {
                if f.r#async
                    || f.generator
                    || f.id.as_ref().is_some_and(|id| {
                        ctx.options().keep_names.function
                            || !ctx.scoping().symbol_is_unused(id.symbol_id())
                    })
                {
                    return;
                }
                (&f.params, true)
            }
            Expression::ArrowFunctionExpression(f) => {
                // Zero-parameter arrows are handled by `substitute_iife_call`.
                if f.r#async || !f.params.has_parameter() {
                    return;
                }
                (&f.params, false)
            }
            _ => return,
        };
        if params.rest.is_some()
            || params
                .items
                .iter()
                .any(|p| !p.pattern.is_binding_identifier() || p.initializer.is_some())
        {
            return;
        }
        let returns_value = match &call.callee {
            Expression::FunctionExpression(f) => {
                let Some(body) = &f.body else { return };
                let Some(returns_value) = Self::single_result_shape(body) else { return };
                returns_value
            }
            Expression::ArrowFunctionExpression(f) => match f.get_function_body() {
                Some(body) => {
                    let Some(returns_value) = Self::single_result_shape(body) else { return };
                    returns_value
                }
                None => true,
            },
            _ => unreachable!(),
        };
        // Decide every parameter's replacement first using O(1) argument and symbol
        // checks before walking the callee body.
        let mut substitutions: Vec<(SymbolId, ConstantValue<'a>)> = vec![];
        let mut substituted_indexes: Vec<usize> = vec![];
        let mut expected_reads: Vec<(SymbolId, usize)> = vec![];
        for (index, param) in params.items.iter().enumerate() {
            let BindingPattern::BindingIdentifier(id) = &param.pattern else { return };
            let symbol_id = id.symbol_id();
            let scoping = ctx.scoping.scoping();
            if !scoping.symbol_redeclarations(symbol_id).is_empty() {
                return;
            }
            let counts = ctx.state.symbols.reference_counts(symbol_id, scoping);
            if counts.has_writes() {
                return;
            }
            let reads = scoping.get_resolved_reference_ids(symbol_id).len();
            if reads == 0 {
                continue;
            }
            let value = match call.arguments.get(index) {
                None => ConstantValue::Undefined,
                Some(argument) => {
                    let argument = argument.to_expression();
                    if !Self::is_inlineable_literal_argument(argument, ctx) {
                        return;
                    }
                    let Some(value) = argument.evaluate_value(ctx) else { return };
                    value
                }
            };
            if reads > 1
                && let ConstantValue::String(s) = &value
                && s.len() > MAX_DUPLICATED_STRING_LEN
            {
                return;
            }
            if matches!(value, ConstantValue::BigInt(_)) {
                return;
            }
            expected_reads.push((symbol_id, reads));
            substitutions.push((symbol_id, value));
            substituted_indexes.push(index);
        }

        if Self::returns_anonymous_function_definition(&call.callee)
            || Self::exposes_caller(call, ctx)
            || Self::names_await_or_yield(&call.callee)
        {
            return;
        }
        let mut checker = ActivationChecker { function_like, found: false };
        match &call.callee {
            Expression::FunctionExpression(f) => {
                checker.visit_function_body(f.body.as_ref().unwrap());
            }
            Expression::ArrowFunctionExpression(f) => {
                checker.visit_arrow_function_body(&f.body);
            }
            _ => unreachable!(),
        }
        if checker.found {
            return;
        }
        for (symbol_id, reads) in expected_reads {
            // The substituter rewrites only expression identifiers; a JSX tag
            // name would be left referencing the dropped parameter.
            let mut counter = ExpressionReferenceCounter { symbol_id, count: 0, ctx };
            counter.visit_expression(&call.callee);
            if counter.count != reads {
                return;
            }
        }
        // Later passes reduce many positions to their side effects, so a lifted
        // returned read could be dropped wherever the call stands.
        if returns_value
            && Self::single_result(&call.callee)
                .is_some_and(|result| Self::discards_observable_read(result, call.span.start, ctx))
        {
            return;
        }
        if call.arguments.iter().any(|argument| {
            Self::discards_observable_read(argument.to_expression(), call.span.start, ctx)
        }) {
            return;
        }

        let span = call.span;
        let pure = call.pure && ctx.annotations();

        // Arguments evaluated for side effects only, in order. Substituted
        // literals and side-effect-free arguments are dropped with the callee.
        let mut prefix = vec![];
        for (index, argument) in call.arguments.iter_mut().enumerate() {
            if substituted_indexes.contains(&index) {
                continue;
            }
            let argument = argument.to_expression_mut();
            if argument.may_have_side_effects(ctx) {
                prefix.push(argument.take_in(ctx));
            }
        }

        let result = match &mut call.callee {
            Expression::FunctionExpression(f) => {
                Self::take_single_result(f.body.as_deref_mut().unwrap(), ctx)
            }
            Expression::ArrowFunctionExpression(f) => match &mut f.body {
                ArrowFunctionBody::FunctionBody(body) => Self::take_single_result(body, ctx),
                body => Some(body.as_expression_mut().unwrap().take_in(ctx)),
            },
            _ => unreachable!(),
        };
        Self::finish_function_iife(
            span,
            result,
            returns_value,
            prefix,
            pure,
            &substitutions,
            e,
            ctx,
        );
    }

    /// Expand an IIFE whose result is unused into its body's expressions.
    ///
    /// `(function () { a(); function f() {} return f })()` → `a()`
    ///
    /// The body may hold only expression statements, empty statements,
    /// function declarations, and a final `return`. Every declaration must be
    /// unreferenced outside the dropped declarations and the side-effect-free
    /// return argument (or the side-effect-free tail of a returned sequence), so the moved expressions never resolve a name to the
    /// discarded scope. Arguments to a parameterless callee run first.
    /// Returns `true` when nothing with side effects remains.
    pub(super) fn expand_unused_function_iife(
        e: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<bool> {
        if ctx.is_tree_shake_only() {
            return None;
        }
        let Expression::CallExpression(call) = e else { return None };
        if call.optional || call.arguments.iter().any(Argument::is_spread) {
            return None;
        }
        let (body, function_like) = match &call.callee {
            Expression::FunctionExpression(f) => {
                if f.r#async
                    || f.generator
                    || f.params.has_parameter()
                    || f.id.as_ref().is_some_and(|id| {
                        ctx.options().keep_names.function
                            || !ctx.scoping().symbol_is_unused(id.symbol_id())
                    })
                {
                    return None;
                }
                (f.body.as_deref()?, true)
            }
            Expression::ArrowFunctionExpression(f) => {
                if f.r#async || f.params.has_parameter() {
                    return None;
                }
                (f.get_function_body()?, false)
            }
            _ => return None,
        };
        if !body.directives.is_empty() || body.statements.is_empty() {
            return None;
        }
        let last = body.statements.len() - 1;
        if body.statements.iter().enumerate().any(|(index, statement)| match statement {
            Statement::ExpressionStatement(_) | Statement::EmptyStatement(_) => false,
            Statement::FunctionDeclaration(f) => f.r#async || f.generator || f.id.is_none(),
            Statement::ReturnStatement(_) => index != last,
            _ => true,
        }) {
            return None;
        }
        if Self::exposes_caller(call, ctx) {
            return None;
        }
        if Self::names_await_or_yield(&call.callee) {
            return None;
        }
        let last = body.statements.len() - 1;
        let mut declarations: Vec<SymbolId> = vec![];
        let mut dropped_return = false;
        let mut checker = ActivationChecker { function_like, found: false };
        for (index, statement) in body.statements.iter().enumerate() {
            match statement {
                Statement::ExpressionStatement(stmt) => checker.visit_expression(&stmt.expression),
                Statement::EmptyStatement(_) => {}
                Statement::FunctionDeclaration(f) => {
                    if f.r#async || f.generator {
                        return None;
                    }
                    declarations.push(f.id.as_ref()?.symbol_id());
                }
                Statement::ReturnStatement(ret) if index == last => match &ret.argument {
                    Some(Expression::SequenceExpression(seq))
                        if !seq.expressions.last().unwrap().may_have_side_effects(ctx) =>
                    {
                        let (tail, kept) = seq.expressions.split_last().unwrap();
                        if Self::discards_observable_read(tail, call.span.start, ctx) {
                            return None;
                        }
                        for expression in kept {
                            checker.visit_expression(expression);
                        }
                        dropped_return = true;
                    }
                    Some(argument) if argument.may_have_side_effects(ctx) => {
                        checker.visit_expression(argument);
                    }
                    Some(argument) => {
                        if Self::discards_observable_read(argument, call.span.start, ctx) {
                            return None;
                        }
                        dropped_return = true;
                    }
                    None => {}
                },
                _ => return None,
            }
            if checker.found {
                return None;
            }
        }
        if !declarations.is_empty() {
            let mut counter =
                DeclarationReferenceCounter { declarations: &declarations, count: 0, ctx };
            for statement in &body.statements {
                match statement {
                    Statement::FunctionDeclaration(f) => {
                        counter.visit_function(f, ScopeFlags::Function);
                    }
                    Statement::ReturnStatement(ret) if dropped_return => match &ret.argument {
                        Some(Expression::SequenceExpression(seq)) => {
                            counter.visit_expression(seq.expressions.last().unwrap());
                        }
                        Some(argument) => counter.visit_expression(argument),
                        None => {}
                    },
                    _ => {}
                }
            }
            let total: usize = declarations
                .iter()
                .map(|symbol_id| ctx.scoping().get_resolved_reference_ids(*symbol_id).len())
                .sum();
            if counter.count != total {
                return None;
            }
        }

        if call.arguments.iter().any(|argument| {
            Self::discards_observable_read(argument.to_expression(), call.span.start, ctx)
        }) {
            return None;
        }

        let span = call.span;
        let mut expressions = vec![];
        for argument in &mut call.arguments {
            let argument = argument.to_expression_mut();
            if argument.may_have_side_effects(ctx) {
                expressions.push(argument.take_in(ctx));
            }
        }
        let body = match &mut call.callee {
            Expression::FunctionExpression(f) => f.body.as_deref_mut().unwrap(),
            Expression::ArrowFunctionExpression(f) => f.get_function_body_mut().unwrap(),
            _ => unreachable!(),
        };
        for statement in &mut body.statements {
            match statement {
                Statement::ExpressionStatement(stmt) => {
                    expressions.push(stmt.expression.take_in(ctx));
                }
                Statement::ReturnStatement(ret) => match &mut ret.argument {
                    Some(Expression::SequenceExpression(seq)) if dropped_return => {
                        let (_, kept) = seq.expressions.split_last_mut().unwrap();
                        expressions.extend(kept.iter_mut().map(|e| e.take_in(ctx)));
                    }
                    Some(argument) if !dropped_return => expressions.push(argument.take_in(ctx)),
                    _ => {}
                },
                _ => {}
            }
        }
        if expressions.is_empty() {
            return Some(true);
        }
        let new_expression = if expressions.len() == 1 {
            expressions.pop().unwrap()
        } else {
            Expression::new_sequence_expression(span, ArenaVec::from_iter_in(expressions, ctx), ctx)
        };
        ctx.replace_expression(e, new_expression);
        Some(false)
    }

    fn take_single_result(
        body: &mut FunctionBody<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        match body.statements.first_mut() {
            None => None,
            Some(Statement::ReturnStatement(ret)) => ret.argument.as_mut().map(|a| a.take_in(ctx)),
            Some(Statement::ExpressionStatement(stmt)) => Some(stmt.expression.take_in(ctx)),
            Some(_) => unreachable!(),
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn finish_function_iife(
        span: Span,
        mut result: Option<Expression<'a>>,
        returns_value: bool,
        prefix: Vec<Expression<'a>>,
        pure: bool,
        substitutions: &[(SymbolId, ConstantValue<'a>)],
        e: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if pure {
            match &mut result {
                Some(Expression::CallExpression(c)) => c.pure = true,
                Some(Expression::NewExpression(n)) => n.pure = true,
                _ => {}
            }
        }
        if let Some(result) = &mut result
            && !substitutions.is_empty()
        {
            let mut substituter = ParameterSubstituter { substitutions, ctx };
            substituter.visit_expression(result);
        }

        let mut expressions = prefix;
        match result {
            Some(result) if returns_value => {
                if expressions.is_empty() {
                    let result = Self::preserve_indirect_access_if_needed(span, result, ctx);
                    ctx.replace_expression(e, result);
                    return;
                }
                expressions.push(result);
            }
            Some(result) => {
                expressions.push(result);
                expressions.push(Expression::new_void_0(span, ctx));
            }
            None => expressions.push(Expression::new_void_0(span, ctx)),
        }
        let new_expression = if expressions.len() == 1 {
            expressions.pop().unwrap()
        } else {
            Expression::new_sequence_expression(span, ArenaVec::from_iter_in(expressions, ctx), ctx)
        };
        ctx.replace_expression(e, new_expression);
    }

    /// Whether discarding `e` unevaluated at source offset `anchor` could hide
    /// one of its reads: a script's top-level binding, a global object property
    /// that an earlier script may have made an accessor, a name a `with` object
    /// may answer with a getter, a lexical binding, import, or parameter that
    /// may still be in its temporal dead zone, where the read throws, or a
    /// derived constructor's `this` that may still precede `super()`.
    /// Later passes drop such a read once it stands alone, so the transform is
    /// skipped instead. Side effects elsewhere in `e` do not keep the read.
    pub(super) fn discards_observable_read(
        e: &Expression<'a>,
        anchor: u32,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        let mut finder = ObservableReadFinder {
            found: false,
            anchor,
            ctx,
            this_may_be_uninitialized: Self::derived_constructor_this_scope(ctx).is_some(),
            in_own_this: false,
        };
        finder.visit_expression(e);
        finder.found
    }

    /// Whether `symbol_id` is a parameter read while its function's parameter
    /// list is still being evaluated, where it may not be initialized yet.
    fn reads_parameter_from_its_list(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        let binding = ctx.scoping().symbol_span(symbol_id);
        let contains_binding = |span: &Span| span.start <= binding.start && binding.end <= span.end;
        ctx.ancestors().any(|ancestor| match ancestor {
            Ancestor::FormalParametersItems(p) => contains_binding(p.span()),
            Ancestor::FormalParametersRest(p) => contains_binding(p.span()),
            _ => false,
        })
    }

    /// Whether a read of the lexical binding `symbol_id` evaluated at `anchor`
    /// may run before the binding is initialized. It runs after only when the
    /// binding is declared earlier in the same function body, outside the
    /// read's own declaration or class, outside the right-hand side of the
    /// `for-in`/`for-of` that declares it, and not in a `switch` whose cases
    /// can jump past its declaration.
    fn may_read_in_tdz(symbol_id: SymbolId, anchor: u32, ctx: &TraverseCtx<'a>) -> bool {
        let scoping = ctx.scoping();
        let declaring_scope = scoping.symbol_scope_id(symbol_id);
        // A read moved out of its binding's scope, such as a class name read
        // in a computed key hoisted out of the class, may have been its own
        // declaration's self-reference.
        if !scoping.scope_ancestors(ctx.current_scope_id()).any(|id| id == declaring_scope) {
            return true;
        }
        let var_scope = |scope_id| {
            scoping.scope_ancestors(scope_id).find(|&id| scoping.scope_flags(id).is_var())
        };
        if var_scope(ctx.current_scope_id()) != var_scope(declaring_scope) {
            return true;
        }
        let binding = scoping.symbol_span(symbol_id);
        if binding.end > anchor {
            return true;
        }
        let contains_binding = |span: &Span| span.start <= binding.start && binding.end <= span.end;
        ctx.ancestors().any(|ancestor| match ancestor {
            Ancestor::VariableDeclaratorId(d) => contains_binding(d.span()),
            Ancestor::VariableDeclaratorInit(d) => contains_binding(d.span()),
            Ancestor::ClassBody(c) => contains_binding(c.span()),
            Ancestor::ClassHeritage(c) => contains_binding(c.span()),
            Ancestor::SwitchStatementCases(s) => s.scope_id().get() == Some(declaring_scope),
            Ancestor::ForInStatementRight(s) => s.scope_id().get() == Some(declaring_scope),
            Ancestor::ForOfStatementRight(s) => s.scope_id().get() == Some(declaring_scope),
            _ => false,
        })
    }

    /// Whether code called from a sloppy-mode IIFE could see it through
    /// `f.caller`, which inlining would change. Strict IIFEs are never visible.
    fn exposes_caller(call: &CallExpression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        let (scope_id, params, body, expression) = match &call.callee {
            Expression::FunctionExpression(f) => (f.scope_id(), &f.params, f.body.as_deref(), None),
            Expression::ArrowFunctionExpression(f) => {
                (f.scope_id(), &f.params, f.get_function_body(), f.get_expression())
            }
            _ => return true,
        };
        let scoping = ctx.scoping();
        // Callees see the function running the call, or nothing if it is strict.
        // A strict IIFE in sloppy code (a class, or `"use strict"`) hides that
        // function until it is inlined.
        let caller_is_strict = scoping
            .scope_ancestors(ctx.current_scope_id())
            .find(|&id| {
                scoping
                    .scope_flags(id)
                    .intersects(ScopeFlags::Function | ScopeFlags::Arrow | ScopeFlags::Top)
            })
            .is_some_and(|id| scoping.scope_flags(id).is_strict_mode());
        if caller_is_strict && scoping.scope_flags(scope_id).is_strict_mode() {
            return false;
        }
        // Inside `with`, any identifier may read an object's getter.
        if scoping
            .scope_ancestors(scope_id)
            .any(|id| scoping.scope_flags(id).contains(ScopeFlags::With))
        {
            return true;
        }
        let outer = CallerInvisibility { params: vec![], ctx };
        let mut primitive_params = vec![];
        for (index, param) in params.items.iter().enumerate() {
            let BindingPattern::BindingIdentifier(id) = &param.pattern else { return true };
            let primitive = match call.arguments.get(index) {
                None => true,
                Some(argument) => argument.as_expression().is_some_and(|a| outer.primitive(a)),
            };
            if primitive && !ctx.state.symbols.symbol_is_mutated(id.symbol_id(), scoping) {
                primitive_params.push(id.symbol_id());
            }
        }
        let checker = CallerInvisibility { params: primitive_params, ctx };
        if let Some(expression) = expression {
            return !checker.safe(expression);
        }
        let Some(body) = body else { return true };
        !body.statements.iter().all(|statement| match statement {
            Statement::ExpressionStatement(stmt) => checker.safe(&stmt.expression),
            Statement::ReturnStatement(ret) => {
                ret.argument.as_ref().is_none_or(|a| checker.safe(a))
            }
            Statement::EmptyStatement(_) | Statement::FunctionDeclaration(_) => true,
            _ => false,
        })
    }

    /// Whether the IIFE returns an anonymous function or class. Put in the
    /// call's place, it would take its `.name` from a binding or key there.
    fn returns_anonymous_function_definition(callee: &Expression<'a>) -> bool {
        let returned = match callee {
            Expression::FunctionExpression(f) => f.body.as_ref().and_then(|b| b.statements.first()),
            Expression::ArrowFunctionExpression(f) => match f.get_expression() {
                Some(expression) => return Self::may_yield_anonymous_function(expression),
                None => f.get_function_body().and_then(|b| b.statements.first()),
            },
            _ => None,
        };
        matches!(returned, Some(Statement::ReturnStatement(ret))
            if ret.argument.as_ref().is_some_and(Self::may_yield_anonymous_function))
    }

    /// Whether the expression, or a branch later folds may reduce it to, is an
    /// anonymous function definition.
    pub(super) fn may_yield_anonymous_function(e: &Expression<'a>) -> bool {
        match e.without_parentheses() {
            Expression::ConditionalExpression(c) => {
                Self::may_yield_anonymous_function(&c.consequent)
                    || Self::may_yield_anonymous_function(&c.alternate)
            }
            Expression::LogicalExpression(l) => {
                Self::may_yield_anonymous_function(&l.left)
                    || Self::may_yield_anonymous_function(&l.right)
            }
            Expression::SequenceExpression(s) => {
                s.expressions.last().is_some_and(Self::may_yield_anonymous_function)
            }
            e => e.is_anonymous_function_definition(),
        }
    }

    /// `Some(returns_value)` when the body is empty, `return [x]`, or one expression statement.
    /// The value an IIFE accepted by `single_result_shape` returns, if any.
    fn single_result<'e>(callee: &'e Expression<'a>) -> Option<&'e Expression<'a>> {
        let statements = match callee {
            Expression::FunctionExpression(f) => &f.body.as_ref()?.statements,
            Expression::ArrowFunctionExpression(f) => match &f.body {
                ArrowFunctionBody::FunctionBody(body) => &body.statements,
                body => return body.as_expression(),
            },
            _ => return None,
        };
        match statements.as_slice() {
            [Statement::ReturnStatement(ret)] => ret.argument.as_ref(),
            _ => None,
        }
    }

    fn single_result_shape(body: &FunctionBody<'a>) -> Option<bool> {
        if !body.directives.is_empty() {
            return None;
        }
        match body.statements.as_slice() {
            [Statement::ReturnStatement(_)] => Some(true),
            [] | [Statement::ExpressionStatement(_)] => Some(false),
            _ => None,
        }
    }

    /// A literal that evaluates to the same primitive wherever it is read.
    fn is_inlineable_literal_argument(argument: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        match argument {
            Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => true,
            Expression::UnaryExpression(unary) => match unary.operator {
                UnaryOperator::Void => !unary.argument.may_have_side_effects(ctx),
                UnaryOperator::UnaryNegation | UnaryOperator::LogicalNot => {
                    matches!(unary.argument, Expression::NumericLiteral(_))
                }
                _ => false,
            },
            Expression::Identifier(ident) => {
                matches!(ident.name.as_str(), "undefined" | "NaN" | "Infinity")
                    && ctx.is_global_reference(ident)
            }
            _ => false,
        }
    }
}

impl<'a> PeepholeOptimizations {
    /// Whether the callee uses `await` or `yield` as a name. Sloppy-mode code
    /// may, but the same text is an operator once moved into an async function
    /// or generator, or a syntax error in a class static block.
    fn names_await_or_yield(callee: &Expression<'a>) -> bool {
        let mut finder = AwaitOrYieldNameFinder { found: false };
        finder.visit_expression(callee);
        finder.found
    }
}

struct AwaitOrYieldNameFinder {
    found: bool,
}

impl AwaitOrYieldNameFinder {
    fn check(&mut self, name: &str) {
        if matches!(name, "await" | "yield") {
            self.found = true;
        }
    }
}

/// Finds a read outside nested functions that
/// `PeepholeOptimizations::discards_observable_read` must keep.
struct ObservableReadFinder<'b, 'a> {
    found: bool,
    anchor: u32,
    ctx: &'b TraverseCtx<'a>,
    /// The visit starts in a derived constructor, where `this` throws before `super()`.
    this_may_be_uninitialized: bool,
    /// Inside an immediately invoked `function`, whose `this` is its own.
    in_own_this: bool,
}

impl<'a> Visit<'a> for ObservableReadFinder<'_, 'a> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let scoping = self.ctx.scoping();
        let Some(reference) = it.reference_id.get().map(|r| scoping.get_reference(r)) else {
            return;
        };
        let symbol_id = reference.symbol_id();
        // A `with` object between the read and its binding may answer it with a getter.
        let declaring_scope = symbol_id.map(|id| scoping.symbol_scope_id(id));
        if scoping
            .scope_ancestors(reference.scope_id())
            .take_while(|&scope_id| Some(scope_id) != declaring_scope)
            .any(|scope_id| scoping.scope_flags(scope_id).contains(ScopeFlags::With))
        {
            self.found = true;
            return;
        }
        // An undeclared name reads a global object property, which an earlier
        // script may have made an accessor, and a standalone `typeof` of it is
        // dropped. Like stock and Terser, `undefined`, `NaN`, `Infinity`, and
        // known built-ins such as `Math` are assumed to be data properties, and
        // so is every global when `unknown_global_side_effects` is off.
        let Some(symbol_id) = symbol_id else {
            if it.may_have_side_effects(self.ctx) {
                self.found = true;
            }
            return;
        };
        let flags = scoping.symbol_flags(symbol_id);
        if (self.ctx.source_type().is_script()
            && scoping.symbol_scope_id(symbol_id) == scoping.root_scope_id())
            || flags.is_import()
            || (flags.is_block_scoped()
                && PeepholeOptimizations::may_read_in_tdz(symbol_id, self.anchor, self.ctx))
            || PeepholeOptimizations::reads_parameter_from_its_list(symbol_id, self.ctx)
        {
            self.found = true;
        }
    }

    // An immediately invoked body runs here, and passes treat it as pure.
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        match it.callee.without_parentheses() {
            Expression::FunctionExpression(f) => {
                if let Some(body) = &f.body {
                    let in_own_this = std::mem::replace(&mut self.in_own_this, true);
                    self.visit_function_body(body);
                    self.in_own_this = in_own_this;
                }
            }
            Expression::ArrowFunctionExpression(f) => self.visit_arrow_function_body(&f.body),
            callee => self.visit_expression(callee),
        }
        self.visit_arguments(&it.arguments);
    }

    fn visit_this_expression(&mut self, _: &ThisExpression) {
        if self.this_may_be_uninitialized && !self.in_own_this {
            self.found = true;
        }
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}

impl<'a> Visit<'a> for AwaitOrYieldNameFinder {
    fn visit_expression(&mut self, it: &Expression<'a>) {
        if !self.found {
            walk::walk_expression(self, it);
        }
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        if !self.found {
            walk::walk_statement(self, it);
        }
    }
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.check(&it.name);
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.check(&it.name);
    }

    fn visit_label_identifier(&mut self, it: &LabelIdentifier<'a>) {
        self.check(&it.name);
    }
}

/// Finds constructs that observe the inlined function's own activation.
struct ActivationChecker {
    /// `true` for `function`, whose `this`/`arguments`/`new.target` are its own.
    function_like: bool,
    found: bool,
}

impl<'a> Visit<'a> for ActivationChecker {
    fn visit_expression(&mut self, it: &Expression<'a>) {
        if !self.found {
            walk::walk_expression(self, it);
        }
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        if !self.found {
            walk::walk_statement(self, it);
        }
    }
    fn visit_this_expression(&mut self, _: &ThisExpression) {
        if self.function_like {
            self.found = true;
        }
    }

    fn visit_super(&mut self, _: &Super) {
        self.found = true;
    }

    fn visit_new_target(&mut self, _: &NewTarget) {
        if self.function_like {
            self.found = true;
        }
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if (self.function_like && it.name == "arguments") || it.name == "eval" {
            self.found = true;
        }
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        // `delete param` is `false`; `delete 1` is `true`.
        if it.operator == UnaryOperator::Delete
            && matches!(it.argument.without_parentheses(), Expression::Identifier(_))
        {
            self.found = true;
        }
        walk::walk_unary_expression(self, it);
    }

    fn visit_class(&mut self, _: &Class<'a>) {
        self.found = true;
    }

    // A `with` object can shadow a parameter read inside its body.
    fn visit_with_statement(&mut self, _: &WithStatement<'a>) {
        self.found = true;
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        // A nested function owns its `this` and `arguments`, but a direct
        // `eval` in it can still read the inlined parameters.
        let function_like = self.function_like;
        self.function_like = false;
        walk::walk_function(self, it, flags);
        self.function_like = function_like;
    }
}

struct ParameterSubstituter<'s, 'c, 'a> {
    substitutions: &'s [(SymbolId, ConstantValue<'a>)],
    ctx: &'c mut TraverseCtx<'a>,
}

impl<'a> VisitMut<'a> for ParameterSubstituter<'_, '_, 'a> {
    fn visit_expression(&mut self, it: &mut Expression<'a>) {
        if let Expression::Identifier(ident) = it
            && let Some(symbol_id) =
                self.ctx.scoping().get_reference(ident.reference_id()).symbol_id()
            && let Some((_, value)) = self.substitutions.iter().find(|(s, _)| *s == symbol_id)
        {
            let scope_id = self.ctx.scoping().get_reference(ident.reference_id()).scope_id();
            self.ctx.state.mark_scope_and_ancestors_dirty(
                scope_id,
                self.ctx.scoping.scoping(),
                false,
            );
            let replacement = self.ctx.value_to_expr_in_scope(it.span(), value.clone(), scope_id);
            self.ctx.replace_expression(it, replacement);
            return;
        }
        walk_mut::walk_expression(self, it);
    }

    fn visit_object_property(&mut self, it: &mut ObjectProperty<'a>) {
        walk_mut::walk_object_property(self, it);
        if it.shorthand && !matches!(it.value, Expression::Identifier(_)) {
            if it.key.is_specific_id("__proto__") {
                // `{ __proto__ }` defines an own property; `{ __proto__: v }`
                // sets the prototype.
                it.computed = true;
                it.key =
                    PropertyKey::new_string_literal(it.key.span(), "__proto__", None, self.ctx);
            }
            it.shorthand = false;
        }
    }
}

/// Counts references to a symbol that appear as identifier expressions.
struct ExpressionReferenceCounter<'c, 'a> {
    symbol_id: SymbolId,
    count: usize,
    ctx: &'c TraverseCtx<'a>,
}

impl<'a> Visit<'a> for ExpressionReferenceCounter<'_, 'a> {
    fn visit_expression(&mut self, it: &Expression<'a>) {
        if let Expression::Identifier(ident) = it {
            if self.ctx.scoping().get_reference(ident.reference_id()).symbol_id()
                == Some(self.symbol_id)
            {
                self.count += 1;
            }
            return;
        }
        walk::walk_expression(self, it);
    }
}

/// Counts references to the dropped declarations inside the dropped code.
struct DeclarationReferenceCounter<'s, 'c, 'a> {
    declarations: &'s [SymbolId],
    count: usize,
    ctx: &'c TraverseCtx<'a>,
}

impl<'a> Visit<'a> for DeclarationReferenceCounter<'_, '_, 'a> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if let Some(symbol_id) = self.ctx.scoping().get_reference(it.reference_id()).symbol_id()
            && self.declarations.contains(&symbol_id)
        {
            self.count += 1;
        }
    }
}

/// Proves that evaluating an expression runs no code but its own: no calls,
/// property or global reads, or conversions of values that may be objects.
struct CallerInvisibility<'b, 'a> {
    /// Parameters known to hold a primitive for the whole call.
    params: Vec<SymbolId>,
    ctx: &'b TraverseCtx<'a>,
}

impl<'a> CallerInvisibility<'_, 'a> {
    /// A binding that is not a property of the global object. At a script's
    /// root, an earlier script may have made it an accessor.
    fn is_local(&self, symbol_id: SymbolId) -> bool {
        let scoping = self.ctx.scoping();
        !(self.ctx.source_type().is_script()
            && scoping.symbol_scope_id(symbol_id) == scoping.root_scope_id())
    }

    fn safe(&self, e: &Expression<'a>) -> bool {
        match e.without_parentheses() {
            Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
            | Expression::FunctionExpression(_)
            | Expression::ArrowFunctionExpression(_) => true,
            Expression::TemplateLiteral(t) => t.expressions.iter().all(|e| self.primitive(e)),
            Expression::Identifier(id) => {
                match self.ctx.scoping().get_reference(id.reference_id()).symbol_id() {
                    Some(symbol_id) => self.is_local(symbol_id),
                    None => {
                        matches!(id.name.as_str(), "undefined" | "NaN" | "Infinity")
                            && self.ctx.is_global_reference(id)
                    }
                }
            }
            Expression::UnaryExpression(u) => match u.operator {
                UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::LogicalNot => {
                    self.safe(&u.argument)
                }
                UnaryOperator::Delete => false,
                _ => self.primitive(&u.argument),
            },
            Expression::BinaryExpression(b) => match b.operator {
                BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => {
                    self.safe(&b.left) && self.safe(&b.right)
                }
                BinaryOperator::In | BinaryOperator::Instanceof => false,
                _ => self.primitive(&b.left) && self.primitive(&b.right),
            },
            Expression::LogicalExpression(l) => self.safe(&l.left) && self.safe(&l.right),
            Expression::ConditionalExpression(c) => {
                self.safe(&c.test) && self.safe(&c.consequent) && self.safe(&c.alternate)
            }
            Expression::SequenceExpression(s) => s.expressions.iter().all(|e| self.safe(e)),
            Expression::ArrayExpression(a) => a.elements.iter().all(|element| match element {
                ArrayExpressionElement::SpreadElement(_) => false,
                ArrayExpressionElement::Elision(_) => true,
                _ => self.safe(element.to_expression()),
            }),
            Expression::ObjectExpression(o) => o.properties.iter().all(|property| match property {
                ObjectPropertyKind::ObjectProperty(p) => {
                    (!p.computed || p.key.as_expression().is_some_and(|k| self.primitive(k)))
                        && self.safe(&p.value)
                }
                ObjectPropertyKind::SpreadProperty(_) => false,
            }),
            Expression::AssignmentExpression(a) => {
                let AssignmentTarget::AssignmentTargetIdentifier(id) = &a.left else {
                    return false;
                };
                let reference = self.ctx.scoping().get_reference(id.reference_id());
                if !reference.symbol_id().is_some_and(|symbol_id| self.is_local(symbol_id)) {
                    return false;
                }
                match a.operator {
                    AssignmentOperator::Assign
                    | AssignmentOperator::LogicalAnd
                    | AssignmentOperator::LogicalOr
                    | AssignmentOperator::LogicalNullish => self.safe(&a.right),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// `safe`, and the result is a primitive.
    fn primitive(&self, e: &Expression<'a>) -> bool {
        match e.without_parentheses() {
            Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::UnaryExpression(_)
            | Expression::BinaryExpression(_) => self.safe(e),
            Expression::Identifier(id) => {
                let reference = self.ctx.scoping().get_reference(id.reference_id());
                match reference.symbol_id() {
                    Some(symbol_id) => self.params.contains(&symbol_id),
                    None => {
                        matches!(id.name.as_str(), "undefined" | "NaN" | "Infinity")
                            && self.ctx.is_global_reference(id)
                    }
                }
            }
            Expression::LogicalExpression(l) => self.primitive(&l.left) && self.primitive(&l.right),
            Expression::ConditionalExpression(c) => {
                self.safe(&c.test) && self.primitive(&c.consequent) && self.primitive(&c.alternate)
            }
            Expression::SequenceExpression(s) => {
                let (last, rest) = s.expressions.split_last().unwrap();
                rest.iter().all(|e| self.safe(e)) && self.primitive(last)
            }
            _ => false,
        }
    }
}

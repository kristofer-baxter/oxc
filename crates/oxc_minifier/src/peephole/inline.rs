use crate::generated::ancestor::Ancestor;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;
use oxc_syntax::symbol::SymbolId;

use crate::TraverseCtx;
use crate::symbol_value::FreshValueKind;

use super::PeepholeOptimizations;

type ParsedEnumMember<'a> = (
    oxc_str::CompactStr,
    ConstantValue<'a>,
    Option<(oxc_str::CompactStr, ConstantValue<'a>)>,
    usize,
);

impl<'a> PeepholeOptimizations {
    /// Seed parameter constants before traversing an immediately invoked function.
    /// The arguments stay at the call site, preserving their evaluation order.
    pub fn init_iife_parameter_values(call: &CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        let params = match &call.callee {
            Expression::ArrowFunctionExpression(arrow) => &arrow.params,
            Expression::FunctionExpression(function) => {
                // Sloppy functions expose mapped arguments and caller introspection.
                // A referenced function name can escape or recurse with other arguments.
                if !ctx.scoping().scope_flags(function.scope_id()).is_strict_mode()
                    || function
                        .id
                        .as_ref()
                        .is_some_and(|id| !ctx.scoping().symbol_is_unused(id.symbol_id()))
                {
                    return;
                }
                &function.params
            }
            _ => return,
        };
        if params.items.is_empty()
            || params.rest.is_some()
            || params
                .items
                .iter()
                .any(|param| !param.pattern.is_binding_identifier() || param.initializer.is_some())
            || call.arguments.iter().any(Argument::is_spread)
        {
            return;
        }
        for (param, argument) in params.items.iter().zip(&call.arguments) {
            let BindingPattern::BindingIdentifier(id) = &param.pattern else { unreachable!() };
            let symbol_id = id.symbol_id();
            // A body declaration can replace a parameter before its first read.
            if !ctx.scoping().symbol_redeclarations(symbol_id).is_empty() {
                continue;
            }
            if let Some(value) = argument.to_expression().evaluate_value(ctx) {
                // `init_value` and its consumers reject direct eval and writes,
                // including writes from nested closures.
                ctx.init_value(symbol_id, Some(value), FreshValueKind::None, false, false)
                    .allow_constant_inlining = false;
                ctx.refresh_symbol_value_digest(symbol_id);
            }
        }
    }

    pub fn init_symbol_value(
        decl: &VariableDeclarator<'a>,
        in_body_statement_list: bool,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Ancestor::VariableDeclarationDeclarations(declaration) = ctx.parent() else {
            unreachable!();
        };
        let declaration_kind = *declaration.kind();
        let BindingPattern::BindingIdentifier(ident) = &decl.id else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        // Evaluate the initializer's constant once; reuse it for the value-context
        // constant and the boolean-falsy fact below. `None` for a non-constant or
        // absent initializer.
        // A string built from a constant that stays behind its binding would copy
        // that constant into every read of this one.
        let init_constant = decl.init.as_ref().and_then(|e| {
            e.evaluate_value(ctx).filter(|value| {
                matches!(e, Expression::StringLiteral(_))
                    || !matches!(value, ConstantValue::String(_))
                    || !Self::reads_non_inlineable_constant(e, ctx)
            })
        });
        let falsy_init = declaration_kind.is_var()
            && init_constant.as_ref().is_some_and(Self::is_falsy_constant);
        let declaration_in_body_statement_list =
            !declaration_kind.is_var() || in_body_statement_list;
        let value = if !in_body_statement_list && Self::is_for_statement_init(ctx) {
            // for-statement initializers have their value set by the for statement itself.
            None
        } else if !declaration_kind.is_var()
            && !in_body_statement_list
            && Self::is_declaration_in_switch_case(ctx)
        {
            // A later case can be entered directly, skipping this initializer, so
            // even reads that follow it in source order may be in its TDZ.
            None
        } else if declaration_kind.is_var() && decl.init.is_some() {
            if init_constant.is_some()
                && Self::is_hoisted_var_inlineable(
                    symbol_id,
                    declaration_in_body_statement_list,
                    ctx,
                )
            {
                init_constant
            } else {
                None
            }
        } else if declaration_kind.is_var()
            && decl.init.is_none()
            && !Self::is_module_scope_symbol(symbol_id, ctx)
        {
            None
        } else {
            decl.init.as_ref().map_or(Some(ConstantValue::Undefined), |_| init_constant)
        };
        let kind = if declaration_in_body_statement_list {
            decl.init.as_ref().map_or(FreshValueKind::None, Self::fresh_value_kind)
        } else {
            FreshValueKind::None
        };
        let implicit_undefined = decl
            .init
            .as_ref()
            .is_none_or(|init| Self::is_direct_implicit_undefined_alias(init, ctx));
        let closure_reads_may_precede_init = !declaration_kind.is_var() && {
            let frame = ctx.state.body_frames.last();
            !(in_body_statement_list
                && ctx.current_scope_id() == frame.scope_id
                && !frame.hoisted_var_inlining_unsafe)
        };
        let declared_end = decl.span.end;
        let symbol_value = ctx.init_value(symbol_id, value, kind, falsy_init, implicit_undefined);
        if closure_reads_may_precede_init {
            symbol_value.tdz_declared_end = Some(declared_end);
        }
        ctx.refresh_symbol_value_digest(symbol_id);
    }

    fn is_declaration_in_switch_case(ctx: &TraverseCtx<'a>) -> bool {
        ctx.ancestors()
            .find(|ancestor| {
                !matches!(
                    ancestor,
                    Ancestor::VariableDeclarationDeclarations(_)
                        | Ancestor::ExportDeclarationDeclaration(_)
                )
            })
            .is_some_and(|ancestor| matches!(ancestor, Ancestor::SwitchCaseConsequent(_)))
    }

    /// Preserve the provenance of an implicit `undefined` through direct
    /// aliases. Materializing the value as `void 0` can be larger than keeping
    /// the alias and can preempt statement-level single-use substitution.
    fn is_direct_implicit_undefined_alias(init: &Expression<'a>, ctx: &TraverseCtx<'a>) -> bool {
        init.get_identifier_reference()
            .and_then(|ident| ctx.scoping().get_reference(ident.reference_id()).symbol_id())
            .and_then(|symbol_id| ctx.state.symbols.value(symbol_id))
            .is_some_and(|value| value.implicit_undefined)
    }

    /// A `ConstantValue` that coerces to `false` (`false`, `0`/`-0`/`NaN`, `""`,
    /// `null`, `undefined`). BigInt is skipped conservatively.
    fn is_falsy_constant(cv: &ConstantValue<'a>) -> bool {
        match cv {
            ConstantValue::Boolean(b) => !b,
            ConstantValue::Number(n) => n.is_nan() || *n == 0.0,
            ConstantValue::String(s) => s.is_empty(),
            ConstantValue::Null | ConstantValue::Undefined => true,
            ConstantValue::BigInt(_) => false,
        }
    }

    /// Whether a declaration is a direct item in the current body rather than
    /// nested in another statement position. Other positions are rejected
    /// conservatively: in particular, brace-less conditional bodies carry no
    /// scope that could reveal that the initializer is optional (#24531).
    pub(super) fn is_declaration_in_body_statement_list(ctx: &TraverseCtx<'a>) -> bool {
        for ancestor in ctx.ancestors() {
            match ancestor {
                Ancestor::VariableDeclarationDeclarations(_)
                | Ancestor::ExportDeclarationDeclaration(_) => {}
                Ancestor::ProgramBody(_) | Ancestor::FunctionBodyStatements(_) => return true,
                _ => return false,
            }
        }
        false
    }

    /// Predicate for inlining a hoisted `var x = <literal>;`. True when no read
    /// can observe `x` as its hoisted `undefined`:
    /// - the declarator sits at the current body's top scope, and that body is
    ///   still in its declarative prelude when any read is inside a closure;
    /// - reads in the same function body fold only when visited after the
    ///   declarator (symbol values reset each pass), so they run after it;
    /// - the declaration is a direct body statement-list item rather than a
    ///   conditional, loop, or other nested statement position;
    /// - it has an initializer (initializer-less vars use the simpler predicate
    ///   below because they are `undefined` both before and after the declaration);
    /// - script-mode top-level vars are excluded (they alias the global object);
    /// - at program scope, if the module loads any other module (`import`,
    ///   `export … from`, `export * from`), skip: a cyclic importer can call
    ///   into our exports and observe any var our exported functions/classes
    ///   close over, regardless of export status;
    /// - closure reads are the gap `substitute_single_use_symbol` can't reach.
    ///   Multiple such reads are
    ///   fine: the prelude check proves none observes the hoisted `undefined`,
    ///   so the value is constant at every read, and the small-value rule /
    ///   write-count guard in `inline_identifier_reference` decide whether each
    ///   read actually folds (e.g. a write-once falsy flag read by `if (flag)`
    ///   throughout — the Svelte/Vue `hydrating` shape, #14001).
    ///
    /// Limitation: the constant is recorded here at the declarator's exit, so a
    /// reader in a function declared *before* the var in source order has
    /// already been visited and won't be inlined. Safe but suboptimal; the
    /// common "flag declared at the top" pattern is unaffected.
    fn is_hoisted_var_inlineable(
        symbol_id: SymbolId,
        declaration_in_body_statement_list: bool,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        if !declaration_in_body_statement_list || Self::is_script_root_scope(ctx) {
            return false;
        }
        // `hoisted_var_inlining_unsafe` is set by a preceding non-declarative
        // statement. The program root additionally starts unsafe when the module
        // has loaders (see `enter_program`), covering the cyclic-import gate.
        let frame = ctx.state.body_frames.last();
        if ctx.current_scope_id() != frame.scope_id {
            return false;
        }
        let mut has_read = false;
        let mut has_closure_read = false;
        for read in ctx.scoping().get_resolved_references(symbol_id).filter(|r| r.is_read()) {
            has_read = true;
            if Self::read_crosses_function_boundary(read.scope_id(), frame.scope_id, ctx) {
                has_closure_read = true;
                break;
            }
        }
        // Reads in the same function body only consume the constant when they
        // are visited after this declarator. Symbol values reset every pass and
        // the declarator is a direct body statement, so such reads come later in
        // source order and run after the initializer. A closure can run earlier,
        // which the declarative prelude rules out.
        has_read && !(has_closure_read && frame.hoisted_var_inlining_unsafe)
    }

    /// Whether a symbol is declared in the root scope of a module.
    fn is_module_scope_symbol(symbol_id: SymbolId, ctx: &TraverseCtx<'a>) -> bool {
        ctx.source_type().is_module()
            && ctx.scoping().symbol_scope_id(symbol_id) == ctx.scoping().root_scope_id()
    }

    /// Classify the fresh value an expression creates (a value that cannot alias
    /// another binding and has no setters/getters that could trigger side effects
    /// on property writes), or `FreshValueKind::None` when it is not fresh.
    pub(super) fn fresh_value_kind(expr: &Expression<'a>) -> FreshValueKind {
        match expr {
            Expression::ArrayExpression(_) => FreshValueKind::Array,
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
                FreshValueKind::Function
            }
            Expression::ObjectExpression(obj) => {
                // Object literals with setter/getter properties are not safe to treat as fresh.
                // Setters trigger side effects on property writes.
                // Getter-only properties throw TypeError in strict mode on write.
                // Also check property values for nested setters/getters.
                let has_side_effects = obj.properties.iter().any(|prop| {
                    matches!(
                        prop,
                        ObjectPropertyKind::ObjectProperty(p)
                            if p.kind.is_accessor()
                                || Self::expression_has_setter_or_getter(&p.value)
                                // `{ __proto__: ... }` sets the prototype chain and could
                                // install setters that make property writes side-effectful.
                                || (p.kind == PropertyKind::Init
                                    && !p.computed
                                    && p.key.is_specific_static_name("__proto__"))
                    )
                });
                if has_side_effects { FreshValueKind::None } else { FreshValueKind::Object }
            }
            Expression::ClassExpression(class) => {
                if Self::class_may_have_property_side_effects(class) {
                    FreshValueKind::None
                } else {
                    FreshValueKind::Class
                }
            }
            _ => FreshValueKind::None,
        }
    }

    /// Check if a class may have side effects on property writes.
    /// Returns `true` if the class has static setters, static accessor properties,
    /// static property definitions with values, or an `extends` clause.
    /// Following SWC's approach: any class with static property definitions
    /// is not considered fresh, because the static initializer runs during
    /// class creation and defines the property via `[[DefineOwnProperty]]`.
    pub(super) fn class_may_have_property_side_effects(class: &Class<'a>) -> bool {
        // Classes with `extends` may inherit static setters from the parent.
        // We can't statically determine the parent's static setters,
        // so conservatively mark as non-fresh.
        if class.heritage.is_some() {
            return true;
        }
        // Class-level decorators run arbitrary code during class creation and can
        // replace the constructor or install setters — never fresh.
        if !class.decorators.is_empty() {
            return true;
        }
        class.body.body.iter().any(|element| match element {
            ClassElement::MethodDefinition(method) => {
                // Any decorator can install a setter or replace the member.
                !method.decorators.is_empty()
                    // A static getter OR setter makes a write to that key throw in
                    // strict mode (a get-only accessor has no [[Set]]; a setter runs
                    // on write), so neither is a droppable fresh-value write.
                    || (method.r#static && method.kind.is_accessor())
            }
            // `static accessor foo` auto-generates a getter+setter pair.
            ClassElement::AccessorProperty(prop) => prop.r#static || !prop.decorators.is_empty(),
            // Any static property definition with a value prevents fresh marking.
            // The value is evaluated during class creation and could interact with
            // property writes in unexpected ways (e.g. nested setters, proxies).
            ClassElement::PropertyDefinition(prop) => {
                !prop.decorators.is_empty() || (prop.r#static && prop.value.is_some())
            }
            // A static block runs arbitrary code during class creation and can
            // install setters or define static properties.
            ClassElement::StaticBlock(_) => true,
            ClassElement::TSIndexSignature(_) => false,
        })
    }

    /// Check if an expression contains setter or getter definitions (recursively).
    pub(super) fn expression_has_setter_or_getter(expr: &Expression<'a>) -> bool {
        match expr {
            Expression::ObjectExpression(obj) => obj.properties.iter().any(|prop| {
                matches!(
                    prop,
                    ObjectPropertyKind::ObjectProperty(p)
                        if p.kind.is_accessor()
                            || Self::expression_has_setter_or_getter(&p.value)
                )
            }),
            Expression::ClassExpression(class) => Self::class_may_have_property_side_effects(class),
            _ => false,
        }
    }

    /// Initialize symbol value for function declarations.
    /// Function declarations always create fresh values (cannot alias another binding).
    pub fn init_function_declaration_symbol_value(
        id: Option<&BindingIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Some(id) = id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        let counts = ctx.state.symbols.reference_counts(symbol_id, ctx.scoping.scoping());
        if counts.has_writes() || counts.has_only_member_write_target_reads() {
            ctx.init_value(symbol_id, None, FreshValueKind::Function, false, false);
            ctx.refresh_symbol_value_digest(symbol_id);
        }
    }

    /// Initialize symbol value for class declarations.
    /// Class declarations create fresh values, but classes with static setters
    /// are not considered fresh because property writes trigger setter side effects.
    pub fn init_class_declaration_symbol_value(class: &Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let Some(id) = &class.id else { return };
        let Some(symbol_id) = id.symbol_id.get() else { return };
        let kind = if Self::class_may_have_property_side_effects(class) {
            FreshValueKind::None
        } else {
            FreshValueKind::Class
        };
        ctx.init_value(symbol_id, None, kind, false, false);
        ctx.refresh_symbol_value_digest(symbol_id);
    }

    fn is_for_statement_init(ctx: &TraverseCtx<'a>) -> bool {
        ctx.ancestor(1).is_parent_of_for_statement_left()
    }

    pub fn inline_identifier_reference(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::Identifier(ident) = expr else { return };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        // `delete x` is `false` for a declared binding, but `delete 7` is `true`.
        if let Ancestor::UnaryExpressionArgument(unary) = ctx.parent()
            && *unary.operator() == UnaryOperator::Delete
        {
            return;
        }
        let Some(symbol_value) = ctx.state.symbols.value(symbol_id) else { return };
        if !symbol_value.can_inline_initialized_constant()
            || !ctx.constant_visible_to_reference(symbol_value, symbol_id, reference_id)
        {
            return;
        }
        let constant = symbol_value.initialized_constant.as_ref().unwrap();
        let new_expr = ctx.value_to_expr(expr.span(), constant.clone());
        ctx.replace_expression(expr, new_expr);
    }

    pub fn collect_program_inlining_facts(program: &Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if ctx.is_tree_shake_only() {
            return;
        }
        if ctx.state.pass_index > 0 && !ctx.state.symbols.verified_enum_iifes.is_empty() {
            let mut newly_empty_enums = Vec::new();
            for (&symbol_id, &(self_iife_ref, self_write_ref)) in
                &ctx.state.symbols.verified_enum_iifes
            {
                if !ctx.state.symbols.enum_iife_symbols.contains(&symbol_id)
                    && ctx
                        .scoping()
                        .get_resolved_reference_ids(symbol_id)
                        .iter()
                        .all(|&r| Some(r) == self_iife_ref || Some(r) == self_write_ref)
                {
                    newly_empty_enums.push(symbol_id);
                }
            }
            for symbol_id in newly_empty_enums {
                ctx.state.symbols.enum_iife_symbols.insert(symbol_id);
                ctx.state.mark_symbol_scopes_dirty(symbol_id, ctx.scoping.scoping(), true);
            }
        }
        let root_scope_id = ctx.scoping().root_scope_id();
        let root_prelude_unsafe = ctx.state.body_frames.last().hoisted_var_inlining_unsafe;
        let allow_enum_iife = if ctx.state.symbols.cached_allow_enum_iife == Some(true) {
            true
        } else {
            let v = !ctx.options().drop_labels.contains("\0no_enum_iife_inlining")
                && !Self::program_may_mutate_object_prototype(program);
            if v {
                ctx.state.symbols.cached_allow_enum_iife = Some(true);
            }
            v
        };
        let mut scanner = InliningCandidateScanner {
            ctx,
            allow_enum_iife,
            obj_candidates: rustc_hash::FxHashMap::default(),
            trivial_functions: Vec::new(),
        };
        if ctx.state.pass_index == 0
            || ctx.state.dirty_all_scopes
            || root_scope_id.index() >= ctx.state.dirty_scopes.capacity()
            || ctx.state.dirty_scopes.has_bit(root_scope_id.index())
        {
            scanner.scan_body_statements(&program.body, root_scope_id, root_prelude_unsafe);
        }
        for stmt in &program.body {
            oxc_ast_visit::Visit::visit_statement(&mut scanner, stmt);
        }
        let mut obj_candidates = scanner.obj_candidates;
        let trivial_functions = scanner.trivial_functions;

        for (symbol_id, decl_end, tdz_end, trivial) in trivial_functions {
            let is_new = ctx
                .state
                .symbols
                .trivial_functions
                .insert(symbol_id, (decl_end, tdz_end, trivial))
                .is_none();
            if is_new && ctx.state.pass_index > 0 {
                ctx.state.mark_symbol_scopes_dirty(symbol_id, ctx.scoping.scoping(), true);
            }
        }

        if obj_candidates.is_empty() {
            return;
        }

        let scoping = ctx.scoping.scoping();
        let mut active_scopes = rustc_hash::FxHashSet::default();
        for (&symbol_id, cand) in &obj_candidates {
            for anc in scoping.scope_ancestors(cand.decl_scope) {
                if !active_scopes.insert(anc) {
                    break;
                }
            }
            for r in scoping.get_resolved_references(symbol_id) {
                for anc in scoping.scope_ancestors(r.scope_id()) {
                    if !active_scopes.insert(anc) {
                        break;
                    }
                }
            }
        }
        let mut verifier =
            NonEscapingObjectVerifier { scoping, candidates: &mut obj_candidates, active_scopes };
        oxc_ast_visit::Visit::visit_program(&mut verifier, program);

        let scoping = ctx.scoping.scoping();
        let mut dirty_symbols = Vec::new();
        for (symbol_id, mut cand) in obj_candidates {
            let total_refs = scoping.get_resolved_reference_ids(symbol_id).len();
            let self_refs = usize::from(cand.self_iife_ref.is_some())
                + usize::from(cand.self_write_ref.is_some());
            if cand.valid_reads + self_refs != total_refs {
                continue;
            }
            if !cand.is_enum_iife && cand.key_reads.len() != cand.props.len() {
                continue;
            }
            // Do not duplicate strings longer than 3 bytes across multiple reads.
            let mut dropped_multi_read_string = false;
            cand.props.retain(|key, val| {
                if let ConstantValue::String(s) = val
                    && s.as_str().is_none_or(|s| s.len() > 3)
                    && cand.key_reads.get(key).copied().unwrap_or(0) > 1
                {
                    dropped_multi_read_string = true;
                    return false;
                }
                true
            });
            if dropped_multi_read_string {
                continue;
            }
            if cand.is_enum_iife {
                ctx.state
                    .symbols
                    .verified_enum_iifes
                    .insert(symbol_id, (cand.self_iife_ref, cand.self_write_ref));
            }
            if cand.is_enum_iife
                && cand.valid_reads == 0
                && ctx.state.symbols.enum_iife_symbols.insert(symbol_id)
                && ctx.state.pass_index > 0
            {
                dirty_symbols.push(symbol_id);
            }
            if cand.valid_reads > 0 {
                let is_new = ctx
                    .state
                    .symbols
                    .member_constants
                    .insert(symbol_id, (cand.decl_end, cand.tdz_end, cand.props))
                    .is_none();
                if is_new && ctx.state.pass_index > 0 {
                    dirty_symbols.push(symbol_id);
                }
            }
        }
        for symbol_id in dirty_symbols {
            ctx.state.mark_symbol_scopes_dirty(symbol_id, ctx.scoping.scoping(), true);
        }
    }

    fn classify_trivial_function(
        params: &FormalParameters<'a>,
        body: Option<&FunctionBody<'a>>,
        expr_body: Option<&Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Option<crate::symbol_state::TrivialFunction<'a>> {
        use crate::symbol_state::TrivialFunction;
        if params.rest.is_some() {
            return None;
        }
        if params.items.is_empty() {
            let ret_expr = if let Some(expr) = expr_body {
                Some(expr)
            } else if let Some(body) = body
                && body.directives.is_empty()
            {
                match body.statements.as_slice() {
                    [Statement::ReturnStatement(ret)] => match &ret.argument {
                        None => {
                            return Some(TrivialFunction::ZeroParamConstant(
                                ConstantValue::Undefined,
                            ));
                        }
                        Some(arg) => Some(arg),
                    },
                    _ => None,
                }
            } else {
                None
            }?;
            if !Self::is_direct_constant_return_expr(ret_expr)
                || ret_expr.may_have_side_effects(ctx)
                || Self::may_yield_anonymous_function(ret_expr)
                || Self::reads_non_inlineable_constant(ret_expr, ctx)
            {
                return None;
            }
            let val = ret_expr.evaluate_value(ctx)?;
            let inlineable = match &val {
                ConstantValue::Number(n) => n.fract() == 0.0 && *n >= -99.0 && *n <= 999.0,
                ConstantValue::BigInt(_) => false,
                ConstantValue::String(s) => s.as_str().is_some_and(|s| s.len() <= 3),
                ConstantValue::Boolean(_) | ConstantValue::Undefined | ConstantValue::Null => true,
            };
            return inlineable.then_some(TrivialFunction::ZeroParamConstant(val));
        }
        if params.items.len() == 1 && params.items[0].initializer.is_none() {
            let BindingPattern::BindingIdentifier(param_id) = &params.items[0].pattern else {
                return None;
            };
            let param_sym = param_id.symbol_id.get()?;
            if ctx.state.symbols.symbol_is_mutated(param_sym, ctx.scoping()) {
                return None;
            }
            let ret_expr = if let Some(expr) = expr_body {
                Some(expr)
            } else if let Some(body) = body
                && body.directives.is_empty()
            {
                match body.statements.as_slice() {
                    [Statement::ReturnStatement(ret)] => ret.argument.as_ref(),
                    [Statement::ReturnStatement(ret), Statement::VariableDeclaration(vd)]
                        if vd.kind.is_var()
                            && vd.declarations.iter().all(|d| {
                                d.init.is_none()
                                    && matches!(
                                        &d.id,
                                        BindingPattern::BindingIdentifier(b)
                                            if b.symbol_id.get() == Some(param_sym)
                                    )
                            }) =>
                    {
                        ret.argument.as_ref()
                    }
                    _ => None,
                }
            } else {
                None
            }?;
            if let Expression::Identifier(ident) = ret_expr.without_parentheses()
                && ctx.scoping().get_reference(ident.reference_id()).symbol_id() == Some(param_sym)
            {
                return Some(TrivialFunction::Identity1Param);
            }
            if ctx.scoping().symbol_is_unused(param_sym)
                && Self::is_direct_constant_return_expr(ret_expr)
                && !ret_expr.may_have_side_effects(ctx)
                && !Self::reads_non_inlineable_constant(ret_expr, ctx)
                && let Some(val @ ConstantValue::Undefined) = ret_expr.evaluate_value(ctx)
            {
                return Some(TrivialFunction::OneParamConstant(val));
            }
        }
        None
    }

    fn is_direct_constant_return_expr(expr: &Expression<'a>) -> bool {
        match expr.without_parentheses() {
            Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => true,
            Expression::Identifier(id) => id.name == "undefined",
            Expression::UnaryExpression(u) => {
                matches!(
                    u.operator,
                    UnaryOperator::UnaryNegation
                        | UnaryOperator::UnaryPlus
                        | UnaryOperator::LogicalNot
                        | UnaryOperator::BitwiseNot
                        | UnaryOperator::Void
                ) && Self::is_direct_constant_return_expr(&u.argument)
            }
            _ => false,
        }
    }

    fn is_reserved_object_prototype_key(key: &str) -> bool {
        matches!(
            key,
            "__proto__"
                | "constructor"
                | "prototype"
                | "toString"
                | "toLocaleString"
                | "valueOf"
                | "hasOwnProperty"
                | "isPrototypeOf"
                | "propertyIsEnumerable"
                | "__defineGetter__"
                | "__defineSetter__"
                | "__lookupGetter__"
                | "__lookupSetter__"
        )
    }

    fn is_prototype_mutation_api_name(name: &str) -> bool {
        matches!(
            name,
            "__proto__"
                | "getPrototypeOf"
                | "defineProperty"
                | "defineProperties"
                | "setPrototypeOf"
                | "__defineSetter__"
                | "__defineGetter__"
                | "eval"
                | "Function"
                | "setTimeout"
                | "setInterval"
                | "execScript"
        )
    }

    pub(crate) fn program_may_mutate_object_prototype(program: &Program<'a>) -> bool {
        struct ObjectProtoScanner {
            found: bool,
        }
        impl<'a> oxc_ast_visit::Visit<'a> for ObjectProtoScanner {
            fn visit_statement(&mut self, it: &Statement<'a>) {
                if !self.found {
                    oxc_ast_visit::walk::walk_statement(self, it);
                }
            }
            fn visit_expression(&mut self, it: &Expression<'a>) {
                if !self.found {
                    oxc_ast_visit::walk::walk_expression(self, it);
                }
            }
            fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
                if matches!(
                    it.name.as_str(),
                    "eval" | "Function" | "setTimeout" | "setInterval" | "execScript"
                ) {
                    self.found = true;
                }
            }
            fn visit_with_statement(&mut self, it: &WithStatement<'a>) {
                self.found = true;
                oxc_ast_visit::walk::walk_with_statement(self, it);
            }
            fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
                let prop = it.property.name.as_str();
                if PeepholeOptimizations::is_prototype_mutation_api_name(prop)
                    || prop == "constructor"
                    || (prop == "prototype"
                        && !matches!(
                            it.object.without_parentheses(),
                            Expression::ClassExpression(_)
                        ))
                {
                    self.found = true;
                }
                oxc_ast_visit::walk::walk_static_member_expression(self, it);
            }
            fn visit_property_key(&mut self, it: &PropertyKey<'a>) {
                if let Some(key) = PeepholeOptimizations::extract_static_member_key(it) {
                    if key == "prototype"
                        || key == "constructor"
                        || PeepholeOptimizations::is_prototype_mutation_api_name(key.as_str())
                    {
                        self.found = true;
                    }
                } else {
                    self.found = true;
                }
                oxc_ast_visit::walk::walk_property_key(self, it);
            }
            fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
                if let Some(s) = it.value.as_str()
                    && (s == "prototype"
                        || s == "constructor"
                        || PeepholeOptimizations::is_prototype_mutation_api_name(s)
                        || s.contains("prototype")
                        || s.contains("__proto__")
                        || s.contains("definePropert")
                        || s.contains("setPrototypeOf")
                        || s.contains("__defineSetter__"))
                {
                    self.found = true;
                }
            }
            fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
                let raw = it.value.raw.as_str();
                if raw == "prototype"
                    || raw == "constructor"
                    || PeepholeOptimizations::is_prototype_mutation_api_name(raw)
                    || raw.contains("prototype")
                    || raw.contains("__proto__")
                    || raw.contains("definePropert")
                    || raw.contains("setPrototypeOf")
                    || raw.contains("__defineSetter__")
                {
                    self.found = true;
                }
            }
            fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
                let is_static_enum_or_literal_key = match it.expression.without_parentheses() {
                    Expression::StringLiteral(_) | Expression::NumericLiteral(_) => true,
                    Expression::AssignmentExpression(assign) => {
                        assign.operator == AssignmentOperator::Assign
                            && match &assign.left {
                                AssignmentTarget::StaticMemberExpression(_) => true,
                                AssignmentTarget::ComputedMemberExpression(cm) => matches!(
                                    cm.expression.without_parentheses(),
                                    Expression::NumericLiteral(_) | Expression::StringLiteral(_)
                                ),
                                _ => false,
                            }
                            && matches!(
                                assign.right.without_parentheses(),
                                Expression::NumericLiteral(_) | Expression::StringLiteral(_)
                            )
                    }
                    _ => false,
                };
                if !is_static_enum_or_literal_key {
                    self.found = true;
                }
                oxc_ast_visit::walk::walk_computed_member_expression(self, it);
            }
        }
        let mut scanner = ObjectProtoScanner { found: false };
        oxc_ast_visit::Visit::visit_program(&mut scanner, program);
        scanner.found
    }

    pub(crate) fn extract_static_member_key(key: &PropertyKey<'a>) -> Option<oxc_str::CompactStr> {
        use oxc_syntax::number::ToJsString;
        match key {
            PropertyKey::StaticIdentifier(id) => Some(oxc_str::CompactStr::from(id.name.as_str())),
            PropertyKey::StringLiteral(s) => s.value.as_str().map(oxc_str::CompactStr::from),
            PropertyKey::NumericLiteral(n) => {
                let s = if n.value == 0.0 { "0".to_string() } else { n.value.to_js_string() };
                Some(oxc_str::CompactStr::from(s))
            }
            _ => None,
        }
    }

    pub(crate) fn extract_computed_member_key(
        expr: &Expression<'a>,
    ) -> Option<oxc_str::CompactStr> {
        use oxc_syntax::number::ToJsString;
        match expr.without_parentheses() {
            Expression::StringLiteral(s) => s.value.as_str().map(oxc_str::CompactStr::from),
            Expression::NumericLiteral(n) => {
                let s = if n.value == 0.0 { "0".to_string() } else { n.value.to_js_string() };
                Some(oxc_str::CompactStr::from(s))
            }
            _ => None,
        }
    }
}

struct ObjectCandidate<'a> {
    decl_scope: oxc_syntax::scope::ScopeId,
    decl_end: u32,
    tdz_end: Option<u32>,
    props: rustc_hash::FxHashMap<oxc_str::CompactStr, ConstantValue<'a>>,
    self_iife_ref: Option<oxc_syntax::reference::ReferenceId>,
    self_write_ref: Option<oxc_syntax::reference::ReferenceId>,
    is_enum_iife: bool,
    valid_reads: usize,
    key_reads: rustc_hash::FxHashMap<oxc_str::CompactStr, u32>,
}

struct InliningCandidateScanner<'c, 'a> {
    ctx: &'c TraverseCtx<'a>,
    allow_enum_iife: bool,
    obj_candidates: rustc_hash::FxHashMap<SymbolId, ObjectCandidate<'a>>,
    trivial_functions: Vec<(SymbolId, u32, Option<u32>, crate::symbol_state::TrivialFunction<'a>)>,
}

impl<'a> InliningCandidateScanner<'_, 'a> {
    fn symbol_is_eligible_for_inlining(
        &self,
        symbol_id: SymbolId,
        body_scope_id: oxc_syntax::scope::ScopeId,
        allow_single_write: Option<oxc_syntax::reference::ReferenceId>,
    ) -> bool {
        let scoping = self.ctx.scoping();
        let decl_scope = scoping.symbol_scope_id(symbol_id);
        if decl_scope != body_scope_id
            || self.ctx.state.symbols.is_implicitly_observable(symbol_id)
            || !scoping.symbol_redeclarations(symbol_id).is_empty()
            || self.ctx.state.hidden_writes.contains(&symbol_id)
            || scoping.scope_flags(decl_scope).contains_direct_eval()
            || (self.ctx.source_type().is_script() && decl_scope == scoping.root_scope_id())
        {
            return false;
        }
        let mut refs = scoping.get_resolved_reference_ids(symbol_id).iter();
        let Some(_) = refs.next() else {
            return false;
        };
        scoping.get_resolved_reference_ids(symbol_id).iter().all(|&ref_id| {
            let r = scoping.get_reference(ref_id);
            let flags_ok = r.flags().is_read_only()
                || (allow_single_write == Some(ref_id) && r.flags().is_write());
            flags_ok
                && !scoping
                    .scope_ancestors(r.scope_id())
                    .take_while(|&s| s != decl_scope)
                    .any(|s| scoping.scope_flags(s).contains(oxc_syntax::scope::ScopeFlags::With))
        })
    }

    fn scan_body_statements(
        &mut self,
        statements: &[Statement<'a>],
        body_scope_id: oxc_syntax::scope::ScopeId,
        mut prelude_unsafe: bool,
    ) {
        let is_script_root = self.ctx.source_type().is_script()
            && body_scope_id == self.ctx.scoping().root_scope_id();
        let mut uninit_vars = rustc_hash::FxHashSet::default();
        for stmt in statements {
            match stmt {
                Statement::FunctionDeclaration(f) => {
                    if !f.r#async
                        && !f.generator
                        && let Some(id) = &f.id
                        && let Some(symbol_id) = id.symbol_id.get()
                        && !self.ctx.state.symbols.trivial_functions.contains_key(&symbol_id)
                        && self.symbol_is_eligible_for_inlining(symbol_id, body_scope_id, None)
                        && let Some(trivial) = PeepholeOptimizations::classify_trivial_function(
                            &f.params,
                            f.body.as_deref(),
                            None,
                            self.ctx,
                        )
                    {
                        self.trivial_functions.push((symbol_id, 0, None, trivial));
                    }
                }
                Statement::VariableDeclaration(decl) if !decl.kind.is_using() => {
                    for d in &decl.declarations {
                        if d.init.is_none()
                            && let BindingPattern::BindingIdentifier(ident) = &d.id
                            && let Some(sym) = ident.symbol_id.get()
                        {
                            uninit_vars.insert(sym);
                        }
                        if !prelude_unsafe {
                            let assigns_global =
                                decl.kind.is_var() && d.init.is_some() && is_script_root;
                            if (!PeepholeOptimizations::is_declarative_variable_declarator(d)
                                || assigns_global)
                                && (is_script_root
                                    || declarator_references_hoisted_or_import(
                                        d,
                                        self.ctx.scoping(),
                                    ))
                            {
                                prelude_unsafe = true;
                            }
                        }
                        self.inspect_declarator(d, decl.kind, body_scope_id, prelude_unsafe);
                    }
                    continue;
                }
                Statement::ExpressionStatement(es) => match es.expression.without_parentheses() {
                    Expression::CallExpression(call) => {
                        self.inspect_statement_enum_iife(
                            call,
                            es.span.end,
                            body_scope_id,
                            prelude_unsafe,
                            &uninit_vars,
                        );
                    }
                    Expression::AssignmentExpression(assign)
                        if assign.operator == AssignmentOperator::Assign =>
                    {
                        if let AssignmentTarget::AssignmentTargetIdentifier(target_id) =
                            &assign.left
                            && let Expression::CallExpression(call) =
                                assign.right.without_parentheses()
                            && call.pure
                        {
                            let ref_id = target_id.reference_id();
                            if let Some(symbol_id) =
                                self.ctx.scoping().get_reference(ref_id).symbol_id()
                                && uninit_vars.contains(&symbol_id)
                                && !self.obj_candidates.contains_key(&symbol_id)
                                && !self.ctx.state.symbols.member_constants.contains_key(&symbol_id)
                                && !self.ctx.state.symbols.enum_iife_symbols.contains(&symbol_id)
                                && self.symbol_is_eligible_for_inlining(
                                    symbol_id,
                                    body_scope_id,
                                    Some(ref_id),
                                )
                            {
                                let is_var = self
                                    .ctx
                                    .scoping()
                                    .symbol_flags(symbol_id)
                                    .is_function_scoped_declaration();
                                let unsafe_var_closure = is_var
                                    && prelude_unsafe
                                    && self.ctx.scoping().get_resolved_references(symbol_id).any(
                                        |r| {
                                            PeepholeOptimizations::read_crosses_function_boundary(
                                                r.scope_id(),
                                                body_scope_id,
                                                self.ctx,
                                            )
                                        },
                                    );
                                if !unsafe_var_closure
                                    && let Some((props, None)) =
                                        self.extract_enum_iife_constants(symbol_id, !is_var, call)
                                {
                                    self.obj_candidates.insert(
                                        symbol_id,
                                        ObjectCandidate {
                                            decl_scope: body_scope_id,
                                            decl_end: es.span.end,
                                            tdz_end: (!is_var && prelude_unsafe)
                                                .then_some(es.span.end),
                                            props,
                                            self_iife_ref: None,
                                            self_write_ref: Some(ref_id),
                                            is_enum_iife: true,
                                            valid_reads: 0,
                                            key_reads: rustc_hash::FxHashMap::default(),
                                        },
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            if !prelude_unsafe
                && !PeepholeOptimizations::is_declarative_body_statement(stmt)
                && (is_script_root
                    || statement_references_hoisted_or_import(stmt, self.ctx.scoping()))
            {
                prelude_unsafe = true;
            }
        }
    }

    fn inspect_statement_enum_iife(
        &mut self,
        call: &CallExpression<'a>,
        decl_end: u32,
        body_scope_id: oxc_syntax::scope::ScopeId,
        prelude_unsafe: bool,
        uninit_vars: &rustc_hash::FxHashSet<SymbolId>,
    ) {
        if call.optional || call.arguments.len() != 1 {
            return;
        }
        let Some(arg_expr) = call.arguments[0].as_expression() else { return };
        let (symbol_id, self_iife_ref, self_write_ref) = match arg_expr.without_parentheses() {
            Expression::LogicalExpression(logic) if logic.operator == LogicalOperator::Or => {
                let Expression::Identifier(read_id) = logic.left.without_parentheses() else {
                    return;
                };
                let Expression::AssignmentExpression(assign) = logic.right.without_parentheses()
                else {
                    return;
                };
                if assign.operator != AssignmentOperator::Assign
                    || !matches!(
                        assign.right.without_parentheses(),
                        Expression::ObjectExpression(obj) if obj.properties.is_empty()
                    )
                {
                    return;
                }
                let AssignmentTarget::AssignmentTargetIdentifier(write_id) = &assign.left else {
                    return;
                };
                let read_ref = read_id.reference_id();
                let write_ref = write_id.reference_id();
                let Some(sym) = self.ctx.scoping().get_reference(read_ref).symbol_id() else {
                    return;
                };
                if self.ctx.scoping().get_reference(write_ref).symbol_id() != Some(sym) {
                    return;
                }
                (sym, Some(read_ref), Some(write_ref))
            }
            Expression::AssignmentExpression(assign)
                if assign.operator == AssignmentOperator::LogicalOr
                    && matches!(
                        assign.right.without_parentheses(),
                        Expression::ObjectExpression(obj) if obj.properties.is_empty()
                    ) =>
            {
                let AssignmentTarget::AssignmentTargetIdentifier(rw_id) = &assign.left else {
                    return;
                };
                let rw_ref = rw_id.reference_id();
                let Some(sym) = self.ctx.scoping().get_reference(rw_ref).symbol_id() else {
                    return;
                };
                (sym, None, Some(rw_ref))
            }
            _ => return,
        };
        if !uninit_vars.contains(&symbol_id)
            || self.obj_candidates.contains_key(&symbol_id)
            || self.ctx.state.symbols.member_constants.contains_key(&symbol_id)
            || self.ctx.state.symbols.enum_iife_symbols.contains(&symbol_id)
            || !self.symbol_is_eligible_for_inlining(symbol_id, body_scope_id, self_write_ref)
        {
            return;
        }
        let is_var = self.ctx.scoping().symbol_flags(symbol_id).is_function_scoped_declaration();
        if is_var
            && prelude_unsafe
            && self.ctx.scoping().get_resolved_references(symbol_id).any(|r| {
                PeepholeOptimizations::read_crosses_function_boundary(
                    r.scope_id(),
                    body_scope_id,
                    self.ctx,
                )
            })
        {
            return;
        }
        if let Some(props) = self.extract_enum_iife_body_constants(call, false) {
            self.obj_candidates.insert(
                symbol_id,
                ObjectCandidate {
                    decl_scope: body_scope_id,
                    decl_end,
                    tdz_end: (!is_var && prelude_unsafe).then_some(decl_end),
                    props,
                    self_iife_ref,
                    self_write_ref,
                    is_enum_iife: true,
                    valid_reads: 0,
                    key_reads: rustc_hash::FxHashMap::default(),
                },
            );
        }
    }

    fn inspect_declarator(
        &mut self,
        d: &VariableDeclarator<'a>,
        kind: VariableDeclarationKind,
        body_scope_id: oxc_syntax::scope::ScopeId,
        prelude_unsafe: bool,
    ) {
        let BindingPattern::BindingIdentifier(ident) = &d.id else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        let Some(init) = &d.init else { return };
        if self.ctx.state.symbols.trivial_functions.contains_key(&symbol_id)
            || self.ctx.state.symbols.member_constants.contains_key(&symbol_id)
            || self.ctx.state.symbols.enum_iife_symbols.contains(&symbol_id)
            || !self.symbol_is_eligible_for_inlining(symbol_id, body_scope_id, None)
        {
            return;
        }
        if kind.is_var()
            && prelude_unsafe
            && self.ctx.scoping().get_resolved_references(symbol_id).any(|r| {
                PeepholeOptimizations::read_crosses_function_boundary(
                    r.scope_id(),
                    body_scope_id,
                    self.ctx,
                )
            })
        {
            return;
        }
        let tdz_end = (!kind.is_var() && prelude_unsafe).then_some(d.span.end);
        let decl_end = d.span.end;

        match init.without_parentheses() {
            Expression::FunctionExpression(f) => {
                if !f.r#async
                    && !f.generator
                    && f.id
                        .as_ref()
                        .is_none_or(|id| self.ctx.scoping().symbol_is_unused(id.symbol_id()))
                    && let Some(trivial) = PeepholeOptimizations::classify_trivial_function(
                        &f.params,
                        f.body.as_deref(),
                        None,
                        self.ctx,
                    )
                {
                    self.trivial_functions.push((symbol_id, decl_end, tdz_end, trivial));
                }
            }
            Expression::ArrowFunctionExpression(a) => {
                if !a.r#async
                    && let Some(trivial) = PeepholeOptimizations::classify_trivial_function(
                        &a.params,
                        a.get_function_body(),
                        a.get_expression(),
                        self.ctx,
                    )
                {
                    self.trivial_functions.push((symbol_id, decl_end, tdz_end, trivial));
                }
            }
            Expression::ObjectExpression(obj) => {
                if let Some(props) = self.extract_plain_object_constants(obj) {
                    self.obj_candidates.insert(
                        symbol_id,
                        ObjectCandidate {
                            decl_scope: body_scope_id,
                            decl_end,
                            tdz_end,
                            props,
                            self_iife_ref: None,
                            self_write_ref: None,
                            is_enum_iife: false,
                            valid_reads: 0,
                            key_reads: rustc_hash::FxHashMap::default(),
                        },
                    );
                }
            }
            Expression::CallExpression(call) => {
                if let Some((props, self_iife_ref)) =
                    self.extract_enum_iife_constants(symbol_id, !kind.is_var(), call)
                {
                    self.obj_candidates.insert(
                        symbol_id,
                        ObjectCandidate {
                            decl_scope: body_scope_id,
                            decl_end,
                            tdz_end,
                            props,
                            self_iife_ref,
                            self_write_ref: None,
                            is_enum_iife: true,
                            valid_reads: 0,
                            key_reads: rustc_hash::FxHashMap::default(),
                        },
                    );
                }
            }
            _ => {}
        }
    }

    fn extract_plain_object_constants(
        &self,
        obj: &ObjectExpression<'a>,
    ) -> Option<rustc_hash::FxHashMap<oxc_str::CompactStr, ConstantValue<'a>>> {
        use oxc_ecmascript::side_effects::MayHaveSideEffects;
        let mut props = rustc_hash::FxHashMap::default();
        for prop in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(p) = prop else {
                return None;
            };
            if p.kind != PropertyKind::Init
                || p.computed
                || PeepholeOptimizations::expression_has_setter_or_getter(&p.value)
                || p.value.may_have_side_effects(self.ctx)
            {
                return None;
            }
            let key_str = PeepholeOptimizations::extract_static_member_key(&p.key)?;
            if PeepholeOptimizations::is_reserved_object_prototype_key(key_str.as_str()) {
                return None;
            }
            if let Some(val) = PeepholeOptimizations::is_direct_constant_return_expr(&p.value)
                .then(|| p.value.evaluate_value(self.ctx))
                .flatten()
            {
                let inlineable = match &val {
                    ConstantValue::Number(_)
                    | ConstantValue::Boolean(_)
                    | ConstantValue::Undefined
                    | ConstantValue::Null => true,
                    ConstantValue::String(s) => {
                        s.as_str().is_some_and(|s| s.len() <= 8)
                            && !PeepholeOptimizations::reads_non_inlineable_constant(
                                &p.value, self.ctx,
                            )
                    }
                    ConstantValue::BigInt(_) => false,
                };
                if inlineable {
                    props.insert(key_str, val);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        (!props.is_empty()).then_some(props)
    }

    fn extract_enum_iife_constants(
        &self,
        enum_symbol_id: SymbolId,
        is_lexical: bool,
        call: &CallExpression<'a>,
    ) -> Option<(
        rustc_hash::FxHashMap<oxc_str::CompactStr, ConstantValue<'a>>,
        Option<oxc_syntax::reference::ReferenceId>,
    )> {
        if call.optional || call.arguments.len() != 1 {
            return None;
        }
        let self_iife_ref = match call.arguments[0].as_expression()?.without_parentheses() {
            Expression::ObjectExpression(obj) if obj.properties.is_empty() => None,
            Expression::LogicalExpression(logic)
                if logic.operator == LogicalOperator::Or
                    && matches!(
                        logic.right.without_parentheses(),
                        Expression::ObjectExpression(obj) if obj.properties.is_empty()
                    ) =>
            {
                let Expression::Identifier(arg_id) = logic.left.without_parentheses() else {
                    return None;
                };
                let ref_id = arg_id.reference_id();
                if is_lexical
                    || self.ctx.scoping().get_reference(ref_id).symbol_id() != Some(enum_symbol_id)
                    || !self.ctx.scoping().get_reference(ref_id).flags().is_read_only()
                {
                    return None;
                }
                Some(ref_id)
            }
            _ => return None,
        };
        let props = self.extract_enum_iife_body_constants(call, true)?;
        Some((props, self_iife_ref))
    }

    /// Extract constant member stores from a TypeScript-style enum IIFE when the
    /// program does not mutate `Object.prototype` and the target keys are not built-in
    /// `Object.prototype` properties (assuming `Object.prototype` is not externally
    /// polluted with custom setters; see `docs/ASSUMPTIONS.md` and
    /// `remove_unused_member_assignment`).
    fn extract_enum_iife_body_constants(
        &self,
        call: &CallExpression<'a>,
        require_return: bool,
    ) -> Option<rustc_hash::FxHashMap<oxc_str::CompactStr, ConstantValue<'a>>> {
        use oxc_ecmascript::side_effects::MayHaveSideEffects;
        if !self.allow_enum_iife {
            return None;
        }
        let (params, body) = match call.callee.without_parentheses() {
            Expression::FunctionExpression(f) if !f.r#async && !f.generator && f.id.is_none() => {
                (&f.params, f.body.as_deref()?)
            }
            Expression::ArrowFunctionExpression(a) if !a.r#async => {
                (&a.params, a.get_function_body()?)
            }
            _ => return None,
        };
        if !body.directives.is_empty()
            || params.rest.is_some()
            || params.items.len() != 1
            || params.items[0].initializer.is_some()
        {
            return None;
        }
        let BindingPattern::BindingIdentifier(param_id) = &params.items[0].pattern else {
            return None;
        };
        let param_sym = param_id.symbol_id.get()?;
        if self.ctx.state.symbols.symbol_is_mutated(param_sym, self.ctx.scoping())
            || !self.ctx.scoping().symbol_redeclarations(param_sym).is_empty()
        {
            return None;
        }
        let mut assign_exprs: Vec<&Expression<'a>> = Vec::new();
        let mut expected_param_refs = 0usize;
        let last_idx = body.statements.len().checked_sub(1)?;
        for (idx, stmt) in body.statements.iter().enumerate() {
            if idx < last_idx {
                let Statement::ExpressionStatement(es) = stmt else { return None };
                match es.expression.without_parentheses() {
                    Expression::SequenceExpression(seq) => {
                        assign_exprs.extend(seq.expressions.iter());
                    }
                    expr => assign_exprs.push(expr),
                }
            } else {
                match stmt {
                    Statement::ReturnStatement(ret) => {
                        let ret_arg = ret.argument.as_ref()?.without_parentheses();
                        match ret_arg {
                            Expression::Identifier(id) => {
                                if self.ctx.scoping().get_reference(id.reference_id()).symbol_id()
                                    != Some(param_sym)
                                {
                                    return None;
                                }
                                expected_param_refs += 1;
                            }
                            Expression::SequenceExpression(seq) => {
                                let (last, prefix) = seq.expressions.split_last()?;
                                let Expression::Identifier(id) = last.without_parentheses() else {
                                    return None;
                                };
                                if self.ctx.scoping().get_reference(id.reference_id()).symbol_id()
                                    != Some(param_sym)
                                {
                                    return None;
                                }
                                expected_param_refs += 1;
                                assign_exprs.extend(prefix.iter());
                            }
                            _ => return None,
                        }
                    }
                    Statement::ExpressionStatement(es) if !require_return => {
                        match es.expression.without_parentheses() {
                            Expression::SequenceExpression(seq) => {
                                assign_exprs.extend(seq.expressions.iter());
                            }
                            expr => assign_exprs.push(expr),
                        }
                    }
                    _ => return None,
                }
            }
        }
        if assign_exprs.is_empty() {
            return None;
        }
        let mut props = rustc_hash::FxHashMap::default();
        for expr in assign_exprs {
            let Expression::AssignmentExpression(assign) = expr.without_parentheses() else {
                return None;
            };
            if assign.operator != AssignmentOperator::Assign {
                return None;
            }
            if let Some((key, val, rev_entry, refs_used)) =
                self.parse_enum_member_assignment(param_sym, assign)
            {
                if PeepholeOptimizations::is_reserved_object_prototype_key(key.as_str()) {
                    return None;
                }
                if assign.right.may_have_side_effects(self.ctx) {
                    return None;
                }
                props.insert(key, val);
                if let Some((rev_key, rev_val)) = rev_entry {
                    if PeepholeOptimizations::is_reserved_object_prototype_key(rev_key.as_str()) {
                        return None;
                    }
                    if let ConstantValue::String(s) = &rev_val
                        && s.as_str().is_some_and(|s| s.len() <= 16)
                    {
                        props.insert(rev_key, rev_val);
                    } else {
                        props.remove(&rev_key);
                    }
                }
                expected_param_refs += refs_used;
            } else {
                return None;
            }
        }
        if self.ctx.scoping().get_resolved_reference_ids(param_sym).len() != expected_param_refs {
            return None;
        }
        (!props.is_empty()).then_some(props)
    }

    fn parse_enum_member_assignment(
        &self,
        param_sym: SymbolId,
        assign: &AssignmentExpression<'a>,
    ) -> Option<ParsedEnumMember<'a>> {
        use oxc_ecmascript::side_effects::MayHaveSideEffects;
        use oxc_syntax::number::ToJsString;
        match &assign.left {
            AssignmentTarget::StaticMemberExpression(m) => {
                if m.optional {
                    return None;
                }
                let Expression::Identifier(obj_id) = m.object.without_parentheses() else {
                    return None;
                };
                if self.ctx.scoping().get_reference(obj_id.reference_id()).symbol_id()
                    != Some(param_sym)
                {
                    return None;
                }
                if !PeepholeOptimizations::is_direct_constant_return_expr(&assign.right) {
                    return None;
                }
                let val = assign.right.evaluate_value(self.ctx)?;
                if !matches!(val, ConstantValue::Number(_) | ConstantValue::String(_)) {
                    return None;
                }
                if let ConstantValue::String(s) = &val
                    && s.as_str().is_none_or(|s| s.len() > 16)
                {
                    return None;
                }
                Some((oxc_str::CompactStr::from(m.property.name.as_str()), val, None, 1))
            }
            AssignmentTarget::ComputedMemberExpression(m) => {
                if m.optional {
                    return None;
                }
                let Expression::Identifier(obj_id) = m.object.without_parentheses() else {
                    return None;
                };
                if self.ctx.scoping().get_reference(obj_id.reference_id()).symbol_id()
                    != Some(param_sym)
                {
                    return None;
                }
                if let Expression::StringLiteral(key_lit) = m.expression.without_parentheses() {
                    if !PeepholeOptimizations::is_direct_constant_return_expr(&assign.right) {
                        return None;
                    }
                    let key_str = key_lit.value.as_str()?;
                    let val = assign.right.evaluate_value(self.ctx)?;
                    if !matches!(val, ConstantValue::Number(_) | ConstantValue::String(_)) {
                        return None;
                    }
                    if let ConstantValue::String(s) = &val
                        && s.as_str().is_none_or(|s| s.len() > 16)
                    {
                        return None;
                    }
                    return Some((oxc_str::CompactStr::from(key_str), val, None, 1));
                }
                // Reverse-mapped numeric/string member: `e[e.Key = <const>] = "Key"`
                let Expression::AssignmentExpression(inner) = m.expression.without_parentheses()
                else {
                    return None;
                };
                if inner.operator != AssignmentOperator::Assign
                    || !PeepholeOptimizations::is_direct_constant_return_expr(&inner.right)
                    || inner.right.may_have_side_effects(self.ctx)
                {
                    return None;
                }
                let inner_key = match &inner.left {
                    AssignmentTarget::StaticMemberExpression(im) if !im.optional => {
                        let Expression::Identifier(inner_obj) = im.object.without_parentheses()
                        else {
                            return None;
                        };
                        if self.ctx.scoping().get_reference(inner_obj.reference_id()).symbol_id()
                            != Some(param_sym)
                        {
                            return None;
                        }
                        im.property.name.as_str()
                    }
                    AssignmentTarget::ComputedMemberExpression(im) if !im.optional => {
                        let Expression::Identifier(inner_obj) = im.object.without_parentheses()
                        else {
                            return None;
                        };
                        if self.ctx.scoping().get_reference(inner_obj.reference_id()).symbol_id()
                            != Some(param_sym)
                        {
                            return None;
                        }
                        let Expression::StringLiteral(s) = im.expression.without_parentheses()
                        else {
                            return None;
                        };
                        s.value.as_str()?
                    }
                    _ => return None,
                };
                let Expression::StringLiteral(outer_rhs) = assign.right.without_parentheses()
                else {
                    return None;
                };
                if outer_rhs.value.as_str() != Some(inner_key) {
                    return None;
                }
                let val = inner.right.evaluate_value(self.ctx)?;
                let rev_key = match &val {
                    ConstantValue::Number(n) => {
                        if *n == 0.0 {
                            oxc_str::CompactStr::from("0")
                        } else {
                            oxc_str::CompactStr::from(n.to_js_string())
                        }
                    }
                    ConstantValue::String(s) => {
                        let str_val = s.as_str()?;
                        if str_val.len() > 16 {
                            return None;
                        }
                        oxc_str::CompactStr::from(str_val)
                    }
                    _ => return None,
                };
                let outer_val = assign.right.evaluate_value(self.ctx)?;
                Some((oxc_str::CompactStr::from(inner_key), val, Some((rev_key, outer_val)), 2))
            }
            _ => None,
        }
    }
}

impl<'a> oxc_ast_visit::Visit<'a> for InliningCandidateScanner<'_, 'a> {
    fn visit_function(&mut self, func: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let Some(scope_id) = func.scope_id.get() {
            if self.ctx.state.should_skip_function_scope(scope_id) {
                return;
            }
            if let Some(body) = &func.body
                && (self.ctx.state.pass_index == 0
                    || self.ctx.state.dirty_all_scopes
                    || scope_id.index() >= self.ctx.state.dirty_scopes.capacity()
                    || self.ctx.state.dirty_scopes.has_bit(scope_id.index()))
            {
                let prelude_unsafe = func.params.has_parameter()
                    && PeepholeOptimizations::declares_parameter_named_function(&func.params, body);
                self.scan_body_statements(&body.statements, scope_id, prelude_unsafe);
            }
        }
        oxc_ast_visit::walk::walk_function(self, func, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(scope_id) = arrow.scope_id.get() {
            if self.ctx.state.should_skip_function_scope(scope_id) {
                return;
            }
            if let Some(body) = arrow.get_function_body()
                && (self.ctx.state.pass_index == 0
                    || self.ctx.state.dirty_all_scopes
                    || scope_id.index() >= self.ctx.state.dirty_scopes.capacity()
                    || self.ctx.state.dirty_scopes.has_bit(scope_id.index()))
            {
                self.scan_body_statements(&body.statements, scope_id, false);
            }
        }
        oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
    }
}

struct NonEscapingObjectVerifier<'s, 'c, 'a> {
    scoping: &'s oxc_semantic::Scoping,
    candidates: &'c mut rustc_hash::FxHashMap<SymbolId, ObjectCandidate<'a>>,
    active_scopes: rustc_hash::FxHashSet<oxc_syntax::scope::ScopeId>,
}

impl<'a> NonEscapingObjectVerifier<'_, '_, 'a> {
    fn check_member_read(&mut self, object: &Expression<'a>, key: Option<oxc_str::CompactStr>) {
        let Some(key) = key else { return };
        let Expression::Identifier(ident) = object.without_parentheses() else { return };
        let ref_id = ident.reference_id();
        let reference = self.scoping.get_reference(ref_id);
        let Some(symbol_id) = reference.symbol_id() else { return };
        let Some(cand) = self.candidates.get_mut(&symbol_id) else { return };
        let crosses_fn = self
            .scoping
            .scope_ancestors(reference.scope_id())
            .take_while(|&s| s != cand.decl_scope)
            .any(|s| self.scoping.scope_flags(s).is_function());
        if !crosses_fn && ident.span.start < cand.decl_end {
            return;
        }
        if cand.props.contains_key(&key) {
            cand.valid_reads += 1;
            *cand.key_reads.entry(key).or_insert(0) += 1;
        }
    }

    fn walk_non_value_member_target(&mut self, expr: &Expression<'a>) {
        match expr.without_parentheses() {
            Expression::StaticMemberExpression(m) => {
                self.visit_expression(&m.object);
            }
            Expression::ComputedMemberExpression(m) => {
                self.visit_expression(&m.object);
                self.visit_expression(&m.expression);
            }
            _ => self.visit_expression(expr),
        }
    }
}

impl<'a> oxc_ast_visit::Visit<'a> for NonEscapingObjectVerifier<'_, '_, 'a> {
    fn visit_function(&mut self, func: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let Some(scope_id) = func.scope_id.get()
            && !self.active_scopes.contains(&scope_id)
        {
            return;
        }
        oxc_ast_visit::walk::walk_function(self, func, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(scope_id) = arrow.scope_id.get()
            && !self.active_scopes.contains(&scope_id)
        {
            return;
        }
        oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
    }

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        match expr {
            Expression::StaticMemberExpression(m) if !m.optional => {
                self.check_member_read(
                    &m.object,
                    Some(oxc_str::CompactStr::from(m.property.name.as_str())),
                );
            }
            Expression::ComputedMemberExpression(m) if !m.optional => {
                self.check_member_read(
                    &m.object,
                    PeepholeOptimizations::extract_computed_member_key(&m.expression),
                );
            }
            _ => {}
        }
        oxc_ast_visit::walk::walk_expression(self, expr);
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        if it.operator.is_delete() {
            self.walk_non_value_member_target(&it.argument);
            return;
        }
        oxc_ast_visit::walk::walk_unary_expression(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.walk_non_value_member_target(&it.callee);
        if let Some(type_args) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_args);
        }
        self.visit_arguments(&it.arguments);
    }

    fn visit_tagged_template_expression(&mut self, it: &TaggedTemplateExpression<'a>) {
        self.walk_non_value_member_target(&it.tag);
        if let Some(type_args) = &it.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_args);
        }
        self.visit_template_literal(&it.quasi);
    }
}

fn statement_references_hoisted_or_import(
    stmt: &Statement<'_>,
    scoping: &oxc_semantic::Scoping,
) -> bool {
    let mut checker = HoistedOrImportRefChecker { scoping, found: false };
    checker.visit_statement(stmt);
    checker.found
}

fn declarator_references_hoisted_or_import(
    decl: &VariableDeclarator<'_>,
    scoping: &oxc_semantic::Scoping,
) -> bool {
    let mut checker = HoistedOrImportRefChecker { scoping, found: false };
    checker.visit_variable_declarator(decl);
    checker.found
}

struct HoistedOrImportRefChecker<'s> {
    scoping: &'s oxc_semantic::Scoping,
    found: bool,
}

impl<'a> oxc_ast_visit::Visit<'a> for HoistedOrImportRefChecker<'_> {
    fn visit_expression(&mut self, it: &Expression<'a>) {
        if !self.found {
            oxc_ast_visit::walk::walk_expression(self, it);
        }
    }

    fn visit_statement(&mut self, it: &Statement<'a>) {
        if !self.found {
            oxc_ast_visit::walk::walk_statement(self, it);
        }
    }

    fn visit_class(&mut self, _: &Class<'a>) {
        self.found = true;
    }

    fn visit_with_statement(&mut self, _: &WithStatement<'a>) {
        self.found = true;
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if it.name == "eval" {
            self.found = true;
            return;
        }
        let Some(symbol_id) =
            it.reference_id.get().and_then(|id| self.scoping.get_reference(id).symbol_id())
        else {
            return;
        };
        if self.scoping.symbol_flags(symbol_id).intersects(
            oxc_syntax::symbol::SymbolFlags::Function
                | oxc_syntax::symbol::SymbolFlags::Class
                | oxc_syntax::symbol::SymbolFlags::Import,
        ) {
            self.found = true;
        }
    }
}

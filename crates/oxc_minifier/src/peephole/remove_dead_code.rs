use crate::generated::ancestor::Ancestor;
use oxc_allocator::{ArenaVec, TakeIn};
use oxc_ast::ast::*;
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::{
    constant_evaluation::{ConstantEvaluation, ConstantValue},
    side_effects::MayHaveSideEffects,
};
use oxc_span::GetSpan;

use crate::{TraverseCtx, keep_var::KeepVar, symbol_metadata::FunctionSummary};

use super::PeepholeOptimizations;

/// Remove Dead Code from the AST.
///
/// Terser option: `dead_code: true`.
///
/// See `KeepVar` at the end of this file for `var` hoisting logic.
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/PeepholeRemoveDeadCode.java>
impl<'a> PeepholeOptimizations {
    /// Remove block from single line blocks
    /// `{ block } -> block`
    pub fn try_optimize_block(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::BlockStatement(s) = stmt else { return };
        match s.body.len() {
            0 => {
                let parent = ctx.parent();
                if parent.is_while_statement()
                    || parent.is_do_while_statement()
                    || parent.is_for_statement()
                    || parent.is_for_in_statement()
                    || parent.is_for_of_statement()
                    || parent.is_block_statement()
                    || parent.is_program()
                {
                    // Remove the block if it is empty and the parent is a block statement.
                    let new_stmt = Statement::new_empty_statement(s.span, ctx);
                    ctx.replace_statement(stmt, new_stmt);
                }
            }
            1 => {
                let first = &s.body[0];
                if matches!(first, Statement::VariableDeclaration(decl) if !decl.kind.is_var())
                    || matches!(first, Statement::ClassDeclaration(_))
                    || matches!(first, Statement::FunctionDeclaration(_))
                    || (matches!(first, Statement::IfStatement(decl) if decl.alternate.is_some())
                        && matches!(ctx.parent(), Ancestor::IfStatementConsequent(_)))
                    || (first.is_iteration_statement() && ctx.parent().is_labeled_statement())
                {
                    return;
                }

                let new_stmt = s.body.remove(0);
                // The unwrap drops no references and preserves side effects,
                // and the enclosing nodes exit later in this pass, so it needs
                // no extra pass on its own. A direct body-list item is the
                // exception: `var` symbol values and the constructor `super()`
                // scan depend on that position, and a later pass sees it.
                if matches!(
                    ctx.parent(),
                    Ancestor::ProgramBody(_) | Ancestor::FunctionBodyStatements(_)
                ) {
                    ctx.replace_statement(stmt, new_stmt);
                } else {
                    ctx.replace_statement_settled(stmt, new_stmt);
                }
            }
            _ => {}
        }
    }

    #[expect(clippy::float_cmp)]
    pub fn try_fold_if(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::IfStatement(if_stmt) = stmt else { return };
        // Annex B gives an unbraced function an implicit block. Moving it into
        // the surrounding statement list can collide with a lexical binding.
        if matches!(&if_stmt.consequent, Statement::FunctionDeclaration(_))
            || matches!(&if_stmt.alternate, Some(Statement::FunctionDeclaration(_)))
        {
            return;
        }
        // Descend and remove `else` blocks first.
        match &mut if_stmt.alternate {
            Some(Statement::IfStatement(_)) => {
                Self::try_fold_if(if_stmt.alternate.as_mut().unwrap(), ctx);
            }
            Some(Statement::BlockStatement(s)) if s.body.is_empty() => {
                if_stmt.alternate = None;
            }
            Some(Statement::EmptyStatement(_)) => {
                if_stmt.alternate = None;
            }
            _ => {}
        }

        if let Some(boolean) = if_stmt.test.evaluate_value_to_boolean(ctx) {
            let test_has_side_effects = if_stmt.test.may_have_side_effects(ctx);
            // Use "1" and "0" instead of "true" and "false" to be shorter.
            // And also prevent swapping consequent and alternate when `!0` is encountered.
            //
            // Idempotency: skip the rewrite when `if_stmt.test` is already the canonical
            // numeric form (NumericLiteral 1.0 / 0.0). Without this gate, the typed
            // `replace_expression` helper would re-record a mutation on every loop
            // iteration (NumericLiteral 1.0 still evaluates to Some(true) via the line-73
            // predicate), preventing fixed-point convergence.
            if !test_has_side_effects
                && !matches!(
                    &if_stmt.test,
                    Expression::NumericLiteral(n)
                        if (boolean && n.value == 1.0) || (!boolean && n.value == 0.0)
                )
            {
                let new_test = Expression::new_numeric_literal(
                    if_stmt.test.span(),
                    if boolean { 1.0 } else { 0.0 },
                    None,
                    NumberBase::Decimal,
                    ctx,
                );
                ctx.replace_expression(&mut if_stmt.test, new_test);
            }
            let mut keep_var = KeepVar::new();
            if boolean {
                if let Some(alternate) = &if_stmt.alternate {
                    keep_var.visit_statement(alternate);
                }
            } else {
                keep_var.visit_statement(&if_stmt.consequent);
            }
            let var_stmt = keep_var
                .get_variable_declaration_statement(ctx)
                .and_then(|stmt| Self::remove_unused_variable_declaration(stmt, ctx));
            let has_var_stmt = var_stmt.is_some();
            if let Some(var_stmt) = var_stmt {
                // Idempotency: skip when the target slot is already in canonical KeepVar
                // output shape (a `var` declaration whose declarators all lack initializers).
                // On the next loop iteration `KeepVar` would re-extract the same names from
                // that slot and produce a structurally-equivalent fresh allocation; routing
                // through `replace_statement` would re-record a mutation indefinitely.
                if boolean {
                    let already_canonical =
                        if_stmt.alternate.as_ref().is_some_and(Self::is_keep_var_canonical);
                    if !already_canonical {
                        if let Some(alternate) = if_stmt.alternate.as_mut() {
                            ctx.replace_statement(alternate, var_stmt);
                        } else {
                            // `KeepVar` only produced a stmt because it visited the alternate,
                            // so the alternate must be Some. Defensive fall-through preserves
                            // historical behaviour: install it without dropping anything.
                            if_stmt.alternate = Some(var_stmt);
                            ctx.notice_change();
                        }
                    }
                } else if !Self::is_keep_var_canonical(&if_stmt.consequent) {
                    ctx.replace_statement(&mut if_stmt.consequent, var_stmt);
                }
                return;
            }
            if test_has_side_effects {
                if !has_var_stmt {
                    if boolean {
                        // Idempotent: `Option::take` only fires when the slot still holds a value.
                        if let Some(old) = if_stmt.alternate.take() {
                            ctx.drop_statement(&old);
                        }
                    } else if !matches!(&if_stmt.consequent, Statement::EmptyStatement(_)) {
                        // Idempotency: skip when consequent is already empty.
                        let new_consequent =
                            Statement::new_empty_statement(if_stmt.consequent.span(), ctx);
                        ctx.replace_statement(&mut if_stmt.consequent, new_consequent);
                    }
                }
                return;
            }
            let new_stmt = if boolean {
                if_stmt.consequent.take_in(ctx)
            } else if let Some(alternate) = if_stmt.alternate.take() {
                alternate
            } else {
                Statement::new_empty_statement(if_stmt.span, ctx)
            };
            ctx.replace_statement(stmt, new_stmt);
        }
    }

    /// True when `stmt` is already in the canonical shape produced by
    /// `KeepVar::get_variable_declaration_statement`: a single `var` declaration
    /// whose declarators all lack initializers. Used as an idempotency gate in
    /// `try_fold_if` so the var-hoisting rewrite doesn't re-fire across loop
    /// iterations when `KeepVar` would just re-emit the same shape.
    fn is_keep_var_canonical(stmt: &Statement<'a>) -> bool {
        matches!(
            stmt,
            Statement::VariableDeclaration(decl)
                if decl.kind.is_var() && decl.declarations.iter().all(|d| d.init.is_none())
        )
    }

    /// `for (init; test; update) break;` → `init, test`
    ///
    /// The body leaves the loop before the update runs, so only the
    /// initializer and one test evaluation remain. Unreachable statements after
    /// the `break` keep their hoisted `var` names; bodies with function
    /// declarations, whose Annex B hoisting is not tracked here, are left alone,
    /// and so is a loop whose test or initializer reads a script global, an
    /// undeclared global, or a lexical binding that may be in its temporal dead
    /// zone.
    pub fn try_fold_for_with_immediate_break(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ForStatement(for_stmt) = stmt else { return };
        let rest = match &for_stmt.body {
            Statement::BreakStatement(b) if b.label.is_none() => &[][..],
            Statement::BlockStatement(block) => match block.body.split_first() {
                Some((Statement::BreakStatement(b), rest)) if b.label.is_none() => rest,
                _ => return,
            },
            _ => return,
        };
        if matches!(&for_stmt.init, Some(ForStatementInit::VariableDeclaration(d)) if !d.kind.is_var())
        {
            return;
        }
        // A test or initializer left standing alone would be dropped later even
        // if it reads a global that an earlier script made an accessor, or a
        // lexical binding still in its temporal dead zone.
        let anchor = for_stmt.span.start;
        let init_expressions: Vec<&Expression<'a>> = match &for_stmt.init {
            Some(ForStatementInit::VariableDeclaration(decl)) => {
                decl.declarations.iter().filter_map(|d| d.init.as_ref()).collect()
            }
            Some(init) => init.as_expression().into_iter().collect(),
            None => vec![],
        };
        if for_stmt
            .test
            .iter()
            .chain(init_expressions)
            .any(|e| Self::discards_observable_read(e, anchor, ctx))
        {
            return;
        }
        let mut detector = FunctionDeclarationDetector { found: false };
        for statement in rest {
            oxc_ast_visit::Visit::visit_statement(&mut detector, statement);
        }
        if detector.found {
            return;
        }
        let mut keep_var = KeepVar::new();
        for statement in rest {
            keep_var.visit_statement(statement);
        }
        ctx.replace_statement_with(stmt, |stmt, ctx| {
            let Statement::ForStatement(for_stmt) = stmt else { unreachable!() };
            let scope_id = for_stmt.scope_id();
            let ForStatement { span, init, test, update, body, .. } = for_stmt.unbox();
            ctx.drop_statement(&body);
            if let Some(update) = update {
                ctx.drop_expression(&update);
            }
            let mut statements = ArenaVec::new_in(ctx);
            let mut expressions = ArenaVec::new_in(ctx);
            match init {
                Some(ForStatementInit::VariableDeclaration(decl)) => {
                    statements.push(Statement::VariableDeclaration(decl));
                }
                Some(init) => expressions.push(init.into_expression()),
                None => {}
            }
            expressions.extend(test);
            if !expressions.is_empty() {
                let expression = if expressions.len() == 1 {
                    expressions.pop().unwrap()
                } else {
                    Expression::new_sequence_expression(span, expressions, ctx)
                };
                statements.push(Statement::new_expression_statement(span, expression, ctx));
            }
            if let Some(var_stmt) = keep_var.get_variable_declaration_statement(ctx) {
                statements.push(var_stmt);
            }
            match statements.len() {
                0 => Statement::new_empty_statement(span, ctx),
                1 => statements.pop().unwrap(),
                _ => Statement::new_block_statement_with_scope_id(span, statements, scope_id, ctx),
            }
        });
    }

    pub fn try_fold_for(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ForStatement(for_stmt) = stmt else { return };
        if let Some(init) = &mut for_stmt.init
            && let Some(init_expr) = init.as_expression_mut()
            && Self::remove_unused_expression(init_expr, ctx)
        {
            ctx.drop_expression(init_expr);
            for_stmt.init = None;
        }
        if let Some(update) = &mut for_stmt.update
            && Self::remove_unused_expression(update, ctx)
        {
            ctx.drop_expression(update);
            for_stmt.update = None;
        }

        let test_boolean =
            for_stmt.test.as_ref().and_then(|test| test.evaluate_value_to_boolean(ctx));
        if for_stmt.test.as_ref().is_some_and(|test| test.may_have_side_effects(ctx)) {
            return;
        }
        match test_boolean {
            Some(false) => match &for_stmt.init {
                Some(ForStatementInit::VariableDeclaration(decl)) if !decl.kind.is_var() => {
                    ctx.replace_statement_with(stmt, |stmt, ctx| {
                        let Statement::ForStatement(for_stmt) = stmt else { unreachable!() };
                        let scope_id = for_stmt.scope_id();
                        let ForStatement { span, init, test, update, body, .. } = for_stmt.unbox();
                        let Some(ForStatementInit::VariableDeclaration(decl)) = init else {
                            unreachable!()
                        };
                        let mut keep_var = KeepVar::new();
                        keep_var.visit_statement(&body);
                        ctx.drop_statement(&body);
                        if let Some(test) = test {
                            ctx.drop_expression(&test);
                        }
                        if let Some(update) = update {
                            ctx.drop_expression(&update);
                        }
                        let mut statements =
                            ArenaVec::from_array_in([Statement::VariableDeclaration(decl)], ctx);
                        if let Some(var_stmt) = keep_var.get_variable_declaration_statement(ctx) {
                            statements.push(var_stmt);
                        }
                        // The initializer still runs, including destructuring effects and TDZ
                        // checks. Reuse the loop scope to keep its lexical bindings isolated.
                        Statement::new_block_statement_with_scope_id(
                            span, statements, scope_id, ctx,
                        )
                    });
                }
                Some(ForStatementInit::VariableDeclaration(_)) => {
                    let mut keep_var = KeepVar::new();
                    keep_var.visit_statement(&for_stmt.body);
                    let mut var_decl = keep_var.get_variable_declaration(ctx);
                    let init_is_var = matches!(
                        &for_stmt.init,
                        Some(ForStatementInit::VariableDeclaration(var_init)) if var_init.kind.is_var()
                    );
                    if init_is_var {
                        if let Some(var_decl) = &mut var_decl {
                            let Some(ForStatementInit::VariableDeclaration(var_init)) =
                                &mut for_stmt.init
                            else {
                                unreachable!()
                            };
                            var_decl.declarations.splice(0..0, var_init.declarations.take_in(ctx));
                        } else {
                            let Some(ForStatementInit::VariableDeclaration(var_init)) =
                                for_stmt.init.take()
                            else {
                                unreachable!()
                            };
                            var_decl = Some(var_init);
                        }
                    }
                    let new_stmt = var_decl.map_or_else(
                        || Statement::new_empty_statement(for_stmt.span, ctx),
                        Statement::VariableDeclaration,
                    );
                    ctx.replace_statement(stmt, new_stmt);
                }
                None => {
                    let mut keep_var = KeepVar::new();
                    keep_var.visit_statement(&for_stmt.body);
                    let new_stmt = keep_var.get_variable_declaration(ctx).map_or_else(
                        || Statement::new_empty_statement(for_stmt.span, ctx),
                        Statement::VariableDeclaration,
                    );
                    ctx.replace_statement(stmt, new_stmt);
                }
                _ => {}
            },
            Some(true) => {
                if let Some(old) = for_stmt.test.take() {
                    ctx.drop_expression(&old);
                }
            }
            None => {}
        }
    }

    /// Remove meaningless labeled statements.
    ///
    /// ```js
    /// a: break a;
    /// ```
    pub fn try_fold_labeled(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::LabeledStatement(s) = stmt else { return };
        let id = s.label.name;

        if ctx.options().drop_labels.contains(id.as_str()) {
            let new_stmt = Statement::new_empty_statement(s.span, ctx);
            ctx.replace_statement(stmt, new_stmt);
            return;
        }

        // Check the first statement in the block, or just the `break [id] ` statement.
        // Check if we need to remove the whole block.
        match &mut s.body {
            Statement::BreakStatement(break_stmt)
                if break_stmt.label.as_ref().is_some_and(|l| l.name == id) => {}
            Statement::BlockStatement(block) if block.body.first().is_some_and(|first| matches!(first, Statement::BreakStatement(break_stmt) if break_stmt.label.as_ref().is_some_and(|l| l.name == id))) => {}
            Statement::EmptyStatement(_) => {
                let new_stmt = Statement::new_empty_statement(s.span, ctx);
                ctx.replace_statement(stmt, new_stmt);
                return;
            }
            _ => return
        }
        let mut var = KeepVar::new();
        var.visit_statement(&s.body);
        let var_decl = var.get_variable_declaration_statement(ctx);
        let new_stmt = var_decl.unwrap_or_else(|| Statement::new_empty_statement(s.span, ctx));
        ctx.replace_statement(stmt, new_stmt);
    }

    pub fn try_fold_expression_stmt(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ExpressionStatement(expr_stmt) = stmt else { return };
        if Self::remove_unused_expression(&mut expr_stmt.expression, ctx) {
            let new_stmt = Statement::new_empty_statement(expr_stmt.span, ctx);
            ctx.replace_statement(stmt, new_stmt);
        }
    }

    pub fn try_fold_switch(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::SwitchStatement(switch_stmt) = stmt else { return };

        // Remove empty case clauses that don't affect behavior.
        // Handles fall-through semantics: remove empty cases before default or at end (if no default).
        // e.g., `switch(x){ case 0: foo(); break; case 1: default: bar() }`
        // => `switch(x){ case 0: foo(); break; default: bar() }`
        // https://github.com/evanw/esbuild/commit/add452ed51333953dd38a26f28a775bb220ea2e9
        let case_count = switch_stmt.cases.len();
        if case_count == 1 {
            if Self::is_switch_case_removable(&switch_stmt.cases[0], true) {
                ctx.drop_switch_case(&switch_stmt.cases.pop().unwrap());
            }
        } else if case_count > 1 {
            // Determine the range [0, end] to check for removable cases.
            // 1. default exists and is empty: check the full switch.
            // 2. default exists and is non-removable and last: check only cases before that default.
            // 3. default exists, is non-removable, and is not last: skip this optimization (`end = 0`).
            // 4. no default case: check the full switch and allow a trailing unlabeled `break`.
            let default_pos = switch_stmt.cases.iter().rposition(SwitchCase::is_default_case);
            let (end, allow_break) = if let Some(default_pos) = default_pos {
                if Self::is_switch_case_removable(&switch_stmt.cases[default_pos], true) {
                    (case_count, true)
                } else if default_pos == case_count - 1 {
                    (default_pos, false)
                } else {
                    (0, false)
                }
            } else {
                (case_count, true)
            };

            if end > 0 {
                // Last non-removable case index in [0, end]. Returns None if all cases are removable.
                let last_non_removable_case_before_end = switch_stmt.cases[..end]
                    .iter()
                    .rposition(|case| !Self::is_switch_case_removable(case, allow_break));

                // Calculate the start of the removable suffix.
                // 1. next case after last non-removable: remove from pos + 1
                // 2. no non-removable case: all cases are removable, start from 0
                let start = match last_non_removable_case_before_end {
                    Some(pos) => pos + 1,
                    None => 0,
                };

                // Remove the removable suffix if any
                if start < end && default_pos.is_none_or(|pos| pos >= start) {
                    for removed_case in switch_stmt.cases.drain(start..end) {
                        ctx.drop_switch_case(&removed_case);
                    }
                }
            }
        }

        if switch_stmt.cases.is_empty() {
            let new_stmt = Statement::new_expression_statement(
                switch_stmt.span,
                switch_stmt.discriminant.take_in(ctx),
                ctx,
            );
            ctx.replace_statement(stmt, new_stmt);
            return;
        }
        if let Some(last_case) = switch_stmt.cases.last_mut()
            && let Some(Statement::BreakStatement(last_break)) = last_case.consequent.last()
            && last_break.label.is_none()
        {
            let dropped = last_case.consequent.pop().unwrap();
            ctx.drop_statement(&dropped);
        }
        if !ctx.is_tree_shake_only() {
            Self::try_merge_or_unwrap_switch(stmt, ctx);
        }
    }

    fn statements_contain_unlabeled_break(stmts: &[Statement<'a>]) -> bool {
        stmts.iter().any(Self::statement_contains_unlabeled_break)
    }

    fn statement_contains_unlabeled_break(stmt: &Statement<'a>) -> bool {
        match stmt {
            Statement::BreakStatement(b) => b.label.is_none(),
            Statement::BlockStatement(b) => Self::statements_contain_unlabeled_break(&b.body),
            Statement::IfStatement(i) => {
                Self::statement_contains_unlabeled_break(&i.consequent)
                    || i.alternate.as_ref().is_some_and(Self::statement_contains_unlabeled_break)
            }
            Statement::TryStatement(t) => {
                Self::statements_contain_unlabeled_break(&t.block.body)
                    || t.handler
                        .as_ref()
                        .is_some_and(|h| Self::statements_contain_unlabeled_break(&h.body.body))
                    || t.finalizer
                        .as_ref()
                        .is_some_and(|f| Self::statements_contain_unlabeled_break(&f.body))
            }
            Statement::LabeledStatement(l) => Self::statement_contains_unlabeled_break(&l.body),
            Statement::WithStatement(w) => Self::statement_contains_unlabeled_break(&w.body),
            _ => false,
        }
    }

    #[expect(clippy::float_cmp, reason = "switch cases use exact JavaScript number equality")]
    fn switch_values_strictly_equal(a: &ConstantValue<'a>, b: &ConstantValue<'a>) -> bool {
        match (a, b) {
            (ConstantValue::Number(na), ConstantValue::Number(nb)) => {
                !na.is_nan() && !nb.is_nan() && na == nb
            }
            _ => a == b,
        }
    }

    #[expect(clippy::float_cmp, reason = "switch cases use exact JavaScript number equality")]
    fn switch_values_may_collide(a: &ConstantValue<'a>, b: &ConstantValue<'a>) -> bool {
        #[cfg(test)]
        tests::COLLISION_COMPARISONS.with(|count| count.set(count.get() + 1));
        match (a, b) {
            (ConstantValue::Number(na), ConstantValue::Number(nb)) => {
                na == nb || (na.is_nan() && nb.is_nan())
            }
            _ => a == b,
        }
    }

    fn switch_body_without_trailing_break<'s>(stmts: &'s [Statement<'a>]) -> &'s [Statement<'a>] {
        if let Some((Statement::BreakStatement(b), prefix)) = stmts.split_last()
            && b.label.is_none()
        {
            prefix
        } else {
            stmts
        }
    }

    fn try_merge_or_unwrap_switch(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        use crate::is_terminated::IsTerminated;
        use oxc_span::ContentEq;

        let Statement::SwitchStatement(switch_stmt) = stmt else { return };
        if switch_stmt
            .cases
            .iter()
            .any(|case| case.consequent.iter().any(Self::statement_cares_about_scope))
        {
            return;
        }

        // Unwrap a single matching case (`switch (1) { case 1: body }` or `switch (x) { default: body }`)
        // when the body has no unlabeled `break`.
        if switch_stmt.cases.len() == 1
            && !Self::statements_contain_unlabeled_break(&switch_stmt.cases[0].consequent)
        {
            let matches_always = match &switch_stmt.cases[0].test {
                None => true,
                Some(test) => {
                    !test.may_have_side_effects(ctx)
                        && !Self::discards_observable_read(test, switch_stmt.span.start, ctx)
                        && !switch_stmt.discriminant.may_have_side_effects(ctx)
                        && switch_stmt
                            .discriminant
                            .evaluate_value(ctx)
                            .zip(test.evaluate_value(ctx))
                            .is_some_and(|(d, t)| Self::switch_values_strictly_equal(&d, &t))
                }
            };
            if matches_always {
                let span = switch_stmt.span;
                let disc = switch_stmt.discriminant.take_in(ctx);
                let mut case = switch_stmt.cases.pop().unwrap();
                if let Some(test) = case.test.take() {
                    ctx.drop_expression(&test);
                }
                let mut body = case.consequent;
                if disc.may_have_side_effects(ctx)
                    || Self::discards_observable_read(&disc, span.start, ctx)
                {
                    body.insert(0, Statement::new_expression_statement(span, disc, ctx));
                } else {
                    ctx.drop_expression(&disc);
                }
                let new_stmt = Statement::new_block_statement_with_scope_id(
                    span,
                    body,
                    ctx.create_child_scope_of_current(oxc_syntax::scope::ScopeFlags::empty()),
                    ctx,
                );
                ctx.replace_statement(stmt, new_stmt);
                return;
            }
        }

        // Merge non-adjacent terminated groups with identical bodies when all case tests
        // are side-effect-free constants (so reordering case labels cannot change behavior).
        let Statement::SwitchStatement(switch_stmt) = stmt else { return };
        if switch_stmt.cases.len() > 1
            && switch_stmt.cases.iter().all(|c| {
                c.test.as_ref().is_none_or(|t| {
                    !t.may_have_side_effects(ctx)
                        && !Self::discards_observable_read(t, switch_stmt.span.start, ctx)
                        && t.evaluate_value(ctx).is_some()
                })
            })
        {
            // Partition cases into fallthrough groups `[start..=end]`.
            let mut groups: Vec<(usize, usize, bool)> = Vec::new();
            let mut group_start = 0;
            for idx in 0..switch_stmt.cases.len() {
                let is_last = idx + 1 == switch_stmt.cases.len();
                if !switch_stmt.cases[idx].consequent.is_empty() || is_last {
                    let terminated = is_last || switch_stmt.cases[idx].consequent.is_terminated();
                    groups.push((group_start, idx, terminated));
                    group_start = idx + 1;
                }
            }
            if groups.len() > 1 && groups.iter().all(|&(_, _, terminated)| terminated) {
                let mut merge_pair = None;
                'outer: for g1 in 0..groups.len() {
                    let (_, end1, _) = groups[g1];
                    let body1 = Self::switch_body_without_trailing_break(
                        &switch_stmt.cases[end1].consequent,
                    );
                    if body1.is_empty() || Self::statements_contain_unlabeled_break(body1) {
                        continue;
                    }
                    for g2 in (g1 + 1)..groups.len() {
                        let (start1, _, _) = groups[g1];
                        let (start2, end2, _) = groups[g2];
                        let body2 = Self::switch_body_without_trailing_break(
                            &switch_stmt.cases[end2].consequent,
                        );
                        if body1.len() != body2.len()
                            || !body1.iter().zip(body2).all(|(a, b)| a.content_eq(b))
                        {
                            continue;
                        }
                        let can_move_across = if g2 == g1 + 1 {
                            true
                        } else {
                            let range1 = &switch_stmt.cases[start1..=end1];
                            let between = &switch_stmt.cases[(end1 + 1)..start2];
                            range1.iter().all(|c1| {
                                let Some(v1) = c1.test.as_ref().and_then(|t| t.evaluate_value(ctx))
                                else {
                                    return false;
                                };
                                between.iter().all(|c2| {
                                    let Some(v2) =
                                        c2.test.as_ref().and_then(|t| t.evaluate_value(ctx))
                                    else {
                                        return false;
                                    };
                                    !Self::switch_values_may_collide(&v1, &v2)
                                })
                            })
                        };
                        if !can_move_across {
                            // Later groups cross the same collision or default case.
                            break;
                        }
                        merge_pair = Some((g1, g2));
                        break 'outer;
                    }
                }
                if let Some((g1, g2)) = merge_pair {
                    let (start1, end1, _) = groups[g1];
                    let (start2, _, _) = groups[g2];
                    for dropped in switch_stmt.cases[end1].consequent.drain(..) {
                        ctx.drop_statement(&dropped);
                    }
                    let moved: Vec<_> = switch_stmt.cases.drain(start1..=end1).collect();
                    let insert_at = start2 - moved.len();
                    for (offset, c) in moved.into_iter().enumerate() {
                        switch_stmt.cases.insert(insert_at + offset, c);
                    }
                    ctx.notice_change();
                }
            }

            // If all cases now fall through into the final case and include `default:`,
            // and the final body has no unlabeled `break`, the switch always runs that body.
            let len = switch_stmt.cases.len();
            if len > 1
                && switch_stmt.cases[..len - 1].iter().all(|c| c.consequent.is_empty())
                && switch_stmt.cases.iter().any(SwitchCase::is_default_case)
                && !Self::statements_contain_unlabeled_break(&switch_stmt.cases[len - 1].consequent)
            {
                let span = switch_stmt.span;
                let disc = switch_stmt.discriminant.take_in(ctx);
                let mut last_case = switch_stmt.cases.pop().unwrap();
                if let Some(test) = last_case.test.take() {
                    ctx.drop_expression(&test);
                }
                let mut body = last_case.consequent;
                for c in switch_stmt.cases.drain(..) {
                    ctx.drop_switch_case(&c);
                }
                if disc.may_have_side_effects(ctx)
                    || Self::discards_observable_read(&disc, span.start, ctx)
                {
                    body.insert(0, Statement::new_expression_statement(span, disc, ctx));
                } else {
                    ctx.drop_expression(&disc);
                }
                let new_stmt = Statement::new_block_statement_with_scope_id(
                    span,
                    body,
                    ctx.create_child_scope_of_current(oxc_syntax::scope::ScopeFlags::empty()),
                    ctx,
                );
                ctx.replace_statement(stmt, new_stmt);
            }
        }
    }

    fn is_switch_case_removable(stmt: &SwitchCase, allow_break: bool) -> bool {
        let is_empty = if stmt.consequent.len() == 1 {
            match stmt.consequent.last() {
                Some(Statement::EmptyStatement(_)) => true,
                Some(Statement::BreakStatement(break_stmt)) => {
                    allow_break && break_stmt.label.is_none()
                }
                _ => false,
            }
        } else {
            stmt.consequent.is_empty()
        };

        is_empty && stmt.test.as_ref().is_none_or(Expression::is_literal)
    }

    pub fn try_fold_try(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::TryStatement(s) = stmt else { return };
        if let Some(handler) = &s.handler
            && s.block.body.is_empty()
        {
            let body = &handler.body.body;
            let is_canonical_body =
                body.is_empty() || (body.len() == 1 && Self::is_keep_var_canonical(&body[0]));
            if !is_canonical_body {
                let mut var = KeepVar::new();
                var.visit_block_statement(&handler.body);
                let Some(handler) = &mut s.handler else { return };

                for dropped in handler.body.body.take_in(ctx) {
                    ctx.drop_statement(&dropped);
                }
                if let Some(var_decl) = var.get_variable_declaration_statement(ctx) {
                    handler.body.body.push(var_decl);
                }
            }
        }

        if let Some(finalizer) = &s.finalizer
            && finalizer.body.is_empty()
            && s.handler.is_some()
        {
            s.finalizer = None;
        }

        if s.block.body.is_empty()
            && s.handler.as_ref().is_none_or(|handler| handler.body.body.is_empty())
        {
            let new_stmt = if let Some(finalizer) = s.finalizer.take() {
                Statement::BlockStatement(finalizer)
            } else {
                Statement::new_empty_statement(s.span, ctx)
            };
            ctx.replace_statement(stmt, new_stmt);
        }
    }

    /// Try folding conditional expression (?:) if the condition results of the condition is known.
    pub fn try_fold_conditional_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ConditionalExpression(e) = expr else { return };
        let Some(v) = e.test.evaluate_value_to_boolean(ctx) else { return };

        ctx.drop_expression(if v { &e.alternate } else { &e.consequent });
        ctx.replace_expression_with(expr, |e, ctx| {
            let Expression::ConditionalExpression(e) = e else {
                unreachable!();
            };
            let mut e = e.unbox();
            let result_expr = if v { e.consequent } else { e.alternate };

            if Self::remove_unused_expression(&mut e.test, ctx) {
                ctx.drop_expression(&e.test);
                // `(1 ? a.b : 0)()` => `(0, a.b)()`
                Self::preserve_indirect_access_if_needed(e.span, result_expr, ctx)
            } else {
                // `(a, true) ? b : c` => `a, b`
                Self::join_sequence(e.span, e.test, result_expr, ctx)
            }
        });
    }

    pub fn remove_sequence_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::SequenceExpression(e) = expr else { return };
        let should_keep_as_sequence_expr = e
            .expressions
            .last()
            .is_some_and(|last_expr| Self::should_keep_indirect_access(last_expr, ctx));
        if should_keep_as_sequence_expr
            && e.expressions.len() == 2
            && e.expressions.first().unwrap().is_number_0()
        {
            return;
        }
        let old_len = e.expressions.len();
        let mut i = 0;
        e.expressions.retain_mut(|e| {
            i += 1;
            if should_keep_as_sequence_expr && i == old_len - 1 {
                // Idempotency: skip the rewrite when `e` is already the canonical
                // `0` placeholder. Without this gate the re-wrap re-bumps the
                // mutation counter every iteration (mirrors the `len == 2` guard
                // above), spinning the fixed-point loop. `0` is side-effect-free,
                // so `remove_unused_expression` would return `true` and produce a
                // structurally-identical fresh `0`.
                if !e.is_number_0() && Self::remove_unused_expression(e, ctx) {
                    let new_expr = Expression::new_number_0(e.span(), ctx);
                    ctx.replace_expression(e, new_expr);
                }
                return true;
            }
            if i == old_len {
                return true;
            }
            if Self::remove_unused_expression(e, ctx) {
                ctx.drop_expression(e);
                false
            } else {
                true
            }
        });
        if e.expressions.len() == 1 {
            let new_expr = e.expressions.pop().unwrap();
            ctx.replace_expression(expr, new_expr);
        }
    }

    pub fn keep_track_of_pure_functions(stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        match stmt {
            Statement::FunctionDeclaration(f) => {
                if let Some(body) = &f.body {
                    Self::try_save_pure_function(
                        f.id.as_ref(),
                        &f.params,
                        f.r#async,
                        f.generator,
                        body.is_empty(),
                        ctx,
                        |ctx| body.statements.iter().any(|stmt| stmt.may_have_side_effects(ctx)),
                    );
                }
            }
            Statement::VariableDeclaration(decl) => {
                for d in &decl.declarations {
                    if let BindingPattern::BindingIdentifier(id) = &d.id {
                        match &d.init {
                            Some(Expression::ArrowFunctionExpression(a)) => {
                                Self::try_save_pure_function(
                                    Some(id),
                                    &a.params,
                                    a.r#async,
                                    false,
                                    a.body.is_empty(),
                                    ctx,
                                    |ctx| {
                                        a.get_expression().map_or_else(
                                            || {
                                                a.get_function_body().is_none_or(|body| {
                                                    body.statements
                                                        .iter()
                                                        .any(|stmt| stmt.may_have_side_effects(ctx))
                                                })
                                            },
                                            |expression| expression.may_have_side_effects(ctx),
                                        )
                                    },
                                );
                            }
                            Some(Expression::FunctionExpression(f)) => {
                                if let Some(body) = &f.body {
                                    Self::try_save_pure_function(
                                        Some(id),
                                        &f.params,
                                        f.r#async,
                                        f.generator,
                                        body.is_empty(),
                                        ctx,
                                        |ctx| {
                                            body.statements
                                                .iter()
                                                .any(|stmt| stmt.may_have_side_effects(ctx))
                                        },
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn try_save_pure_function(
        id: Option<&BindingIdentifier<'a>>,
        params: &FormalParameters<'a>,
        r#async: bool,
        generator: bool,
        returns_undefined: bool,
        ctx: &mut TraverseCtx<'a>,
        body_has_side_effects: impl FnOnce(&TraverseCtx<'a>) -> bool,
    ) {
        if r#async || generator {
            return;
        }
        let Some(symbol_id) = id.and_then(|id| id.symbol_id.get()) else { return };
        let binding_scope_id = ctx.scoping().symbol_scope_id(symbol_id);
        let binding_scope_flags = ctx.scoping().scope_flags(binding_scope_id);
        if !ctx.scoping().symbol_redeclarations(symbol_id).is_empty()
            || ctx.state.hidden_writes.contains(&symbol_id)
            || binding_scope_flags.contains_direct_eval()
            || (ctx.source_type().is_script() && binding_scope_id == ctx.scoping().root_scope_id())
        {
            if ctx.state.symbols.clear_function_summary(symbol_id) {
                ctx.state.mark_symbol_scopes_dirty(
                    symbol_id,
                    ctx.scoping.scoping(),
                    ctx.state.pass_index > 0,
                );
            }
            return;
        }
        if ctx.scoping().symbol_is_unused(symbol_id)
            || !ctx.scoping().get_resolved_references(symbol_id).all(|r| r.flags().is_read_only())
        {
            return;
        }
        let summary = if returns_undefined {
            FunctionSummary::SideEffectFreeReturnsUndefined
        } else {
            FunctionSummary::SideEffectFree
        };
        if ctx.state.symbols.function_summary(symbol_id) == summary {
            return;
        }
        // Destructuring can throw. Default initializers run for missing or `undefined`
        // arguments, and function summaries are call-independent, so reject an initializer
        // that may have side effects. TDZ-only throws follow the minifier's documented
        // `No TDZ Violation` assumption.
        if !params.items.iter().all(|param| {
            param.pattern.is_binding_identifier()
                && param.initializer.as_ref().is_none_or(|init| !init.may_have_side_effects(ctx))
        }) || params
            .rest
            .as_ref()
            .is_some_and(|rest| !rest.rest.argument.is_binding_identifier())
        {
            return;
        }
        if body_has_side_effects(ctx) {
            return;
        }
        if ctx.state.symbols.set_function_summary(symbol_id, summary) {
            ctx.state.mark_symbol_scopes_dirty(
                symbol_id,
                ctx.scoping.scoping(),
                ctx.state.pass_index > 0,
            );
        }
    }

    pub fn remove_dead_code_call_expression(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::CallExpression(e) = expr else { return };
        if let Expression::Identifier(ident) = &e.callee {
            let reference_id = ident.reference_id();
            let reference = ctx.scoping().get_reference(reference_id);
            if let Some(symbol_id) = reference.symbol_id() {
                if !ctx.is_tree_shake_only()
                    && !e.optional
                    && let Some((decl_end, tdz_end, trivial)) =
                        ctx.state.symbols.trivial_functions.get(&symbol_id).cloned()
                    && ctx.inlining_visible_to_reference(
                        decl_end,
                        tdz_end,
                        symbol_id,
                        reference_id,
                        ident.span.start,
                    )
                {
                    match trivial {
                        crate::symbol_state::TrivialFunction::ZeroParamConstant(val)
                            if e.arguments.is_empty() =>
                        {
                            let new_expr = ctx.value_to_expr(e.span, val);
                            ctx.replace_expression(expr, new_expr);
                            return;
                        }
                        crate::symbol_state::TrivialFunction::OneParamConstant(val)
                            if e.arguments.len() <= 1
                                && e.arguments.first().is_none_or(|a| {
                                    a.as_expression().is_some_and(|arg_expr| {
                                        !arg_expr.may_have_side_effects(ctx)
                                            && !Self::discards_observable_read(
                                                arg_expr,
                                                e.span.start,
                                                ctx,
                                            )
                                    })
                                }) =>
                        {
                            let new_expr = ctx.value_to_expr(e.span, val);
                            ctx.replace_expression(expr, new_expr);
                            return;
                        }
                        crate::symbol_state::TrivialFunction::Identity1Param
                            if e.arguments.len() <= 1
                                && e.arguments.first().is_none_or(|a| !a.is_spread()) =>
                        {
                            if e.arguments.is_empty() {
                                let new_expr = Expression::new_void_0(e.span, ctx);
                                ctx.replace_expression(expr, new_expr);
                                return;
                            }
                            let arg_expr = e.arguments[0].to_expression();
                            let allow_anon_fn = matches!(
                                ctx.parent(),
                                Ancestor::CallExpressionCallee(_)
                                    | Ancestor::CallExpressionArguments(_)
                            );
                            if (allow_anon_fn || !Self::may_yield_anonymous_function(arg_expr))
                                && !Self::discards_observable_read(arg_expr, e.span.start, ctx)
                            {
                                let arg = e.arguments.pop().unwrap().into_expression();
                                let new_expr =
                                    Self::preserve_indirect_access_if_needed(e.span, arg, ctx);
                                ctx.replace_expression(expr, new_expr);
                                return;
                            }
                        }
                        _ => {}
                    }
                }
                if ctx.state.symbols.function_summary(symbol_id).returns_undefined() {
                    let mut exprs =
                        Self::fold_arguments_into_needed_expressions(&mut e.arguments, ctx);
                    if exprs.is_empty() {
                        let new_expr = Expression::new_void_0(e.span, ctx);
                        ctx.replace_expression(expr, new_expr);
                        return;
                    }
                    exprs.push(Expression::new_void_0(e.span, ctx));
                    let new_expr = Expression::new_sequence_expression(e.span, exprs, ctx);
                    ctx.replace_expression(expr, new_expr);
                }
            }
        }
    }

    /// Whether the indirect access should be kept.
    /// For example, `(0, foo.bar)()` should not be transformed to `foo.bar()`.
    /// Example case: `let o = { f() { assert.ok(this !== o); } }; (true && o.f)(); (true && o.f)``;`
    ///
    /// * `access_value` - The expression that may need to be kept as indirect reference (`foo.bar` in the example above)
    pub fn should_keep_indirect_access(
        access_value: &Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> bool {
        match ctx.parent() {
            Ancestor::CallExpressionCallee(_) | Ancestor::TaggedTemplateExpressionTag(_) => {
                match access_value {
                    Expression::Identifier(id) => id.name == "eval" && ctx.is_global_reference(id),
                    match_member_expression!(Expression) => true,
                    // Example case: `(0, o?.f)()` (`this` is undefined) -> `(o?.f)()` (`this` is `o`)
                    Expression::ChainExpression(chain) => {
                        matches!(&chain.expression, match_member_expression!(ChainElement))
                    }
                    _ => false,
                }
            }
            Ancestor::UnaryExpressionArgument(unary) => match unary.operator() {
                UnaryOperator::Typeof => {
                    // Example case: `typeof (0, foo)` (error) -> `typeof foo` (no error)
                    if let Expression::Identifier(id) = access_value {
                        ctx.is_global_reference(id)
                    } else {
                        false
                    }
                }
                UnaryOperator::Delete => {
                    match access_value {
                        // Example case: `delete (0, foo)` (no error) -> `delete foo` (error)
                        Expression::Identifier(_)
                        // Example case: `delete (0, foo.#a)` (no error) -> `delete foo.#a` (error)
                        | Expression::PrivateFieldExpression(_)
                        // Example case: `delete (0, foo.bar)` (noop) -> `delete foo.bar` (deletes bar)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::StaticMemberExpression(_) => true,
                        // Example case: `delete (0, foo?.bar)` (noop) -> `delete foo?.bar` (deletes bar)
                        Expression::ChainExpression(chain) => {
                            matches!(&chain.expression, match_member_expression!(ChainElement))
                        }
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Wrap `expr` as `(0, expr)` so an access that
    /// [`Self::should_keep_indirect_access`] flagged stays indirect.
    ///
    /// The shape is load-bearing: `remove_sequence_expression`'s idempotency
    /// gate recognizes exactly a 2-element sequence whose head `is_number_0`.
    /// Sibling fold sites in `fold_constants.rs` / `try_fold_conditional_expression`
    /// still build it inline; migrating them here is welcome.
    pub fn preserve_indirect_access(
        span: Span,
        expr: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        Expression::new_sequence_expression(
            span.merge(expr.span()),
            [Expression::new_number_0(span, ctx), expr],
            ctx,
        )
    }

    /// Wrap `expr` as `(0, expr)` only if [`Self::should_keep_indirect_access`] flags it.
    pub fn preserve_indirect_access_if_needed(
        span: Span,
        expr: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        if Self::should_keep_indirect_access(&expr, ctx) {
            Self::preserve_indirect_access(span, expr, ctx)
        } else {
            expr
        }
    }

    pub fn remove_dead_code_exit_class_body(body: &mut ClassBody<'a>, _ctx: &mut TraverseCtx<'a>) {
        body.body.retain(|e| !matches!(e, ClassElement::StaticBlock(s) if s.body.is_empty()));
    }

    pub fn remove_empty_spread_arguments(args: &mut ArenaVec<'a, Argument<'a>>) {
        if args.len() != 1 {
            return;
        }
        let Argument::SpreadElement(e) = &args[0] else { return };
        let Expression::ArrayExpression(e) = &e.argument else { return };
        if e.elements.is_empty() {
            args.drain(..);
        }
    }
}

/// Finds function declarations, including Annex B block-level ones.
struct FunctionDeclarationDetector {
    found: bool,
}

impl<'a> oxc_ast_visit::Visit<'a> for FunctionDeclarationDetector {
    fn visit_function(&mut self, it: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
        if it.is_declaration() {
            self.found = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::fmt::Write;

    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    use crate::{CompressOptions, Minifier, MinifierOptions};

    thread_local! {
        pub(super) static COLLISION_COMPARISONS: Cell<usize> = const { Cell::new(0) };
    }

    #[test]
    fn distinct_switch_bodies_do_not_scan_case_collisions() {
        let mut source = String::from("switch (value) {");
        for index in 0..100 {
            write!(source, "case {index}: handler{index}(); break;")
                .expect("writing to a String cannot fail");
        }
        source.push('}');
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let mut program = parsed.program;
        COLLISION_COMPARISONS.with(|count| count.set(0));
        Minifier::new(MinifierOptions {
            compress: Some(CompressOptions::smallest()),
            mangle: None,
            ..MinifierOptions::default()
        })
        .minify(&allocator, &mut program);
        COLLISION_COMPARISONS.with(|count| {
            assert_eq!(
                count.get(),
                0,
                "different bodies cannot merge regardless of case collisions"
            );
        });
    }
}

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::{ast::*, builder::GetAstBuilder};
use oxc_ast_visit::VisitJs;
use oxc_ecmascript::BoundNames;
use oxc_semantic::Scoping;
use oxc_span::{SPAN, Span};
use oxc_str::Ident;
use oxc_syntax::{
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};

use crate::TraverseCtx;

pub struct KeepVar<'a> {
    vars: Vec<(Ident<'a>, Span, Option<SymbolId>)>,
    /// Function declarations in the dropped code, with the function's own scope.
    /// A sloppy-mode one nested in a block also binds a `var` of its name in the
    /// enclosing function (Annex B.3.3), which semantic analysis records by
    /// moving its symbol into that var scope.
    functions: Vec<(Ident<'a>, Span, SymbolId, ScopeId)>,
}

impl<'a> VisitJs<'a> for KeepVar<'a> {
    fn visit_statement(&mut self, it: &Statement<'a>) {
        // Only visit blocks where vars could be hoisted
        match it {
            Statement::BlockStatement(it) => self.visit_block_statement(it),
            Statement::BreakStatement(it) => self.visit_break_statement(it),
            Statement::ContinueStatement(it) => self.visit_continue_statement(it),
            // Statement::DebuggerStatement(it) => self.visit_debugger_statement(it),
            Statement::DoWhileStatement(it) => self.visit_do_while_statement(it),
            // Statement::EmptyStatement(it) => self.visit_empty_statement(it),
            // Statement::ExpressionStatement(it) => self.visit_expression_statement(it),
            Statement::ForInStatement(it) => self.visit_for_in_statement(it),
            Statement::ForOfStatement(it) => self.visit_for_of_statement(it),
            Statement::ForStatement(it) => self.visit_for_statement(it),
            Statement::IfStatement(it) => self.visit_if_statement(it),
            Statement::LabeledStatement(it) => self.visit_labeled_statement(it),
            // Statement::ReturnStatement(it) => self.visit_return_statement(it),
            Statement::SwitchStatement(it) => self.visit_switch_statement(it),
            // Statement::ThrowStatement(it) => self.visit_throw_statement(it),
            Statement::TryStatement(it) => self.visit_try_statement(it),
            Statement::WhileStatement(it) => self.visit_while_statement(it),
            Statement::WithStatement(it) => self.visit_with_statement(it),
            // match_declaration!(Statement) => visitor.visit_declaration(it.to_declaration()),
            // match_module_declaration!(Statement) => {
            // visitor.visit_module_declaration(it.to_module_declaration())
            // }
            Statement::VariableDeclaration(decl) => self.visit_variable_declaration(decl),
            Statement::FunctionDeclaration(func) => {
                if let (Some(id), Some(scope_id)) = (&func.id, func.scope_id.get())
                    && let Some(symbol_id) = id.symbol_id.get()
                {
                    self.functions.push((id.name, id.span, symbol_id, scope_id));
                }
            }
            _ => {}
        }
    }

    // A nested function's (or class's) `var`s and block functions bind in its
    // own scope, not in the scope that receives the hoisted declaration.
    fn visit_function(&mut self, _it: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _it: &Class<'a>) {}

    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        if it.kind.is_var() {
            it.bound_names(&mut |ident| {
                self.vars.push((ident.name, ident.span, ident.symbol_id.get()));
            });
        }
    }
}

impl<'a> KeepVar<'a> {
    pub fn new() -> Self {
        Self { vars: std::vec![], functions: std::vec![] }
    }

    /// Whether dropping a function declaration also drops a function-scoped
    /// binding that a `var` must keep. That is the Annex B.3.3 binding of a
    /// sloppy-mode block function, or a function that is the direct body of an
    /// `if` or label. Dead code never assigns it, so `var name;` keeps it
    /// `undefined` instead of letting reads escape to an outer binding.
    fn keeps_function_scoped_binding(
        scoping: &Scoping,
        name: Ident<'a>,
        symbol_id: SymbolId,
        function_scope_id: ScopeId,
    ) -> bool {
        let symbol_scope_id = scoping.symbol_scope_id(symbol_id);
        if !scoping.scope_flags(symbol_scope_id).is_var()
            || scoping.symbol_flags(symbol_id).intersects(SymbolFlags::BlockScoped)
        {
            return false;
        }
        let Some(declaring_scope_id) = scoping.scope_parent_id(function_scope_id) else {
            return false;
        };
        // A `var` cannot be declared through a lexical binding of the same name,
        // and Annex B skips the function then too.
        !scoping
            .scope_ancestors(declaring_scope_id)
            .take_while(|&scope_id| scope_id != symbol_scope_id)
            .any(|scope_id| scoping.get_binding(scope_id, name).is_some())
    }

    pub fn get_variable_declaration(
        mut self,
        ctx: &TraverseCtx<'a>,
    ) -> Option<ArenaBox<'a, VariableDeclaration<'a>>> {
        let scoping = ctx.scoping();
        let functions = std::mem::take(&mut self.functions);
        for &(name, span, symbol_id, function_scope_id) in &functions {
            if Self::keeps_function_scoped_binding(scoping, name, symbol_id, function_scope_id) {
                self.vars.push((name, span, Some(symbol_id)));
            }
        }
        if self.vars.is_empty() {
            return None;
        }

        let builder = ctx.builder();

        let kind = VariableDeclarationKind::Var;
        let decls = ArenaVec::from_iter_in(
            self.vars.into_iter().map(|(name, span, symbol_id)| {
                let id = symbol_id.map_or_else(
                    || BindingPattern::new_binding_identifier(span, name, builder),
                    |symbol_id| {
                        BindingPattern::new_binding_identifier_with_symbol_id(
                            span, name, symbol_id, builder,
                        )
                    },
                );
                VariableDeclarator::new(span, id, None, None, false, builder)
            }),
            builder,
        );

        Some(VariableDeclaration::boxed(SPAN, kind, decls, false, builder))
    }

    pub fn get_variable_declaration_statement(
        self,
        ctx: &TraverseCtx<'a>,
    ) -> Option<Statement<'a>> {
        self.get_variable_declaration(ctx).map(Statement::VariableDeclaration)
    }
}

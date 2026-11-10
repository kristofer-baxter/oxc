//! Whether a statement can run a closure declared in the current body.
//!
//! A hoisted `var x = <literal>` can be read as `undefined` only by a closure
//! that runs before the initializer. A closure can run only after code creates
//! it (a function, arrow, or class in the statement) or names it (a function or
//! class binding). A function declaration that shares a parameter's name is
//! the exception: `arguments` holds it, so its body starts unsafe (see
//! `declares_parameter_named_function`). Code outside the body cannot reach a closure that has not
//! escaped, so a statement without those shapes keeps the body's declarative
//! prelude open even when it calls globals, reads getters, or throws. A
//! script's root is the exception: its function declarations are properties of
//! the global object, which any global call can reach, so callers treat every
//! non-declarative statement there as exposing, and so is a `var` initializer
//! there, which can run a setter that an earlier script defined. Terser's
//! `reduce_vars` reasons the same way: it walks a function declaration only
//! where it is referenced.

use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolFlags;

pub(super) fn statement_may_expose_closure(stmt: &Statement<'_>, scoping: &Scoping) -> bool {
    let mut checker = ClosureExposureChecker { scoping, found: false };
    checker.visit_statement(stmt);
    checker.found
}

pub(super) fn declarator_may_expose_closure(
    decl: &VariableDeclarator<'_>,
    scoping: &Scoping,
) -> bool {
    let mut checker = ClosureExposureChecker { scoping, found: false };
    checker.visit_variable_declarator(decl);
    checker.found
}

struct ClosureExposureChecker<'s> {
    scoping: &'s Scoping,
    found: bool,
}

impl<'a> Visit<'a> for ClosureExposureChecker<'_> {
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

    fn visit_function(&mut self, _: &Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
        self.found = true;
    }

    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {
        self.found = true;
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
        if self
            .scoping
            .symbol_flags(symbol_id)
            .intersects(SymbolFlags::Function | SymbolFlags::Class | SymbolFlags::Import)
        {
            self.found = true;
        }
    }
}

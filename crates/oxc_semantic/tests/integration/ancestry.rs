use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

/// `leave_node` without a matching `enter_node` must not read outside the ancestry stack.
#[test]
fn leave_node_on_empty_ancestry_stack() {
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, "null;", SourceType::mjs()).parse().program;
    let kind = AstKind::Program(&program);

    for check_syntax_error in [false, true] {
        let mut builder = SemanticBuilder::new()
            .with_build_nodes(false)
            .with_check_syntax_error(check_syntax_error);
        builder.leave_node(kind);
        builder.leave_node(kind);
    }

    // The builder still works after the unbalanced calls.
    let mut builder = SemanticBuilder::new_compiler();
    builder.leave_node(kind);
    let ret = builder.build(&program);
    assert!(ret.diagnostics.is_empty());
}

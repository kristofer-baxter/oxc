use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Statement};
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn minify(source: &str, source_type: SourceType, keep_names: bool) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions {
        compress: None,
        mangle: Some(MangleOptions {
            top_level: Some(true),
            keep_names: MangleOptionsKeepNames { function: keep_names, class: false },
            ..MangleOptions::default()
        }),
        ..MinifierOptions::default()
    })
    .minify(&allocator, &mut program);
    Codegen::new().with_scoping(result.scoping).build(&program).code
}

#[test]
fn observable_function_names_are_reserved_but_call_only_names_can_shorten() {
    let output = minify(
        "function visible() {} use(visible.name); function callOnly() {} callOnly(); const inferred = () => {}; use(inferred.name);",
        SourceType::mjs(),
        true,
    );
    assert!(output.contains("function visible("), "{output}");
    assert!(output.contains("const inferred ="), "{output}");
    assert!(!output.contains("callOnly"), "{output}");
}

#[test]
fn with_intercepted_binding_names_are_not_changed() {
    let output = minify(
        "function outer(obj) { let shadowed = 1; with (obj) { use(shadowed); } return shadowed; }",
        SourceType::script(),
        false,
    );
    assert!(output.contains("let shadowed = 1"), "{output}");
    assert!(output.contains("use(shadowed)"), "{output}");
    assert!(output.contains("return shadowed"), "{output}");
}

#[test]
fn commonjs_wrapper_aliases_are_not_renamed() {
    let output = minify("var module = 1; arguments[2] = 2; use(module);", SourceType::cjs(), false);
    assert!(output.contains("var module = 1"), "{output}");
    assert!(output.contains("use(module)"), "{output}");
}

#[test]
fn annex_b_block_function_and_hoisted_binding_share_a_mangled_name() {
    let output = minify(
        "function outer() { var value = 1; { function value() {} } return value; } use(outer());",
        SourceType::script(),
        false,
    );
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &output, SourceType::script()).parse();
    assert!(parsed.diagnostics.is_empty());
    let Statement::FunctionDeclaration(outer) = &parsed.program.body[0] else { panic!("{output}") };
    let statements = &outer.body.as_ref().unwrap().statements;
    let Statement::VariableDeclaration(variable) = &statements[0] else { panic!("{output}") };
    let BindingPattern::BindingIdentifier(binding) = &variable.declarations[0].id else {
        panic!("{output}")
    };
    let Statement::BlockStatement(block) = &statements[1] else { panic!("{output}") };
    let Statement::FunctionDeclaration(inner) = &block.body[0] else { panic!("{output}") };
    assert_eq!(binding.name, inner.id.as_ref().unwrap().name, "{output}");
}

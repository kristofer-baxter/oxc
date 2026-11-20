use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

fn run(enum_eval: bool) {
    let source_text = "enum E { A = 1 }";
    let source_type = SourceType::ts();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let scoping =
        SemanticBuilder::new().with_enum_eval(enum_eval).build(&program).semantic.into_scoping();
    let _ret = Transformer::new(&allocator, Path::new(""), &TransformOptions::default())
        .build_with_scoping(scoping, &mut program);
}

#[test]
fn transforms_enum_when_enum_eval_enabled() {
    run(true);
}

#[test]
#[should_panic(expected = "SemanticBuilder::with_enum_eval(true)")]
#[cfg(debug_assertions)]
fn panics_when_enum_eval_disabled() {
    run(false);
}

#[test]
fn non_finite_members_do_not_read_shadowed_globals() {
    let source_text = "const Infinity = 1, NaN = 2;\n\
        enum E { A = 1 / 0, B = -1 / 0, C = 0 / 0, D = A }\n\
        export const x = E.D;";
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::ts()).parse();
    let mut program = ret.program;
    let scoping =
        SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.into_scoping();
    let ret = Transformer::new(&allocator, Path::new(""), &TransformOptions::default())
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.is_empty());
    let code = oxc_codegen::Codegen::new().build(&program).code;
    assert_eq!(code.matches("Infinity").count(), 1, "{code}");
    assert_eq!(code.matches("NaN").count(), 1, "{code}");
    assert!(code.contains("E[\"A\"] = 1 / 0"), "{code}");
    assert!(code.contains("E[\"B\"] = -1 / 0"), "{code}");
    assert!(code.contains("E[\"C\"] = 0 / 0"), "{code}");
}

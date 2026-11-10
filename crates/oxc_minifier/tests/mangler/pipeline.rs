use std::process::Command;

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Statement};
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn minify(source: &str, source_type: SourceType, keep_names: bool) -> String {
    minify_with_compression(source, source_type, keep_names, None)
}

fn minify_with_compression(
    source: &str,
    source_type: SourceType,
    keep_names: bool,
    compress: Option<CompressOptions>,
) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut program = parsed.program;
    let result = Minifier::new(MinifierOptions {
        compress,
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

fn stack_function_name(source: &str, source_type: SourceType) -> String {
    let output = Command::new("node")
        .args([
            "--input-type",
            if source_type.is_module() { "module" } else { "commonjs" },
            "--eval",
            source,
        ])
        .output()
        .expect("Node.js is required to verify function names in stack traces");
    assert!(output.status.success(), "Node.js failed: {}", String::from_utf8_lossy(&output.stderr));
    let stack = String::from_utf8(output.stdout).expect("Node.js stack output must be UTF-8");
    stack
        .lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().nth(1))
        .expect("Error stack must contain a named first frame")
        .to_owned()
}

#[test]
fn function_names_are_kept_even_when_only_called_directly() {
    let output = minify(
        "function visible() {} use(visible.name); function callOnly() {} callOnly(); const inferred = () => {}; use(inferred.name);",
        SourceType::mjs(),
        true,
    );
    assert!(output.contains("function visible("), "{output}");
    assert!(output.contains("const inferred ="), "{output}");
    assert!(output.contains("function callOnly("), "{output}");
}

#[test]
fn stack_function_names_are_kept_in_modules_and_nested_strict_scripts() {
    for (source, source_type) in [
        (
            "function descriptive() { return new Error().stack; } console.log(descriptive());",
            SourceType::mjs(),
        ),
        (
            "function outer() { 'use strict'; function descriptive() { return new Error().stack; } console.log(descriptive()); } outer();",
            SourceType::script(),
        ),
    ] {
        let original_name = stack_function_name(source, source_type);
        assert_eq!(original_name, "descriptive", "{source}");
        for compress in [None, Some(CompressOptions::safest())] {
            let output = minify_with_compression(source, source_type, true, compress);
            assert!(output.contains("function descriptive("), "{output}");
            assert_eq!(stack_function_name(&output, source_type), original_name, "{output}");
        }
        let output = minify(source, source_type, false);
        assert!(!output.contains("descriptive"), "{output}");
        assert_ne!(stack_function_name(&output, source_type), original_name, "{output}");
    }
}

#[test]
fn inferred_call_only_function_names_are_kept() {
    for (source, name) in [
        ("const inferred = () => new Error().stack; console.log(inferred());", "inferred"),
        (
            "function outer(fallback = () => new Error().stack) { return fallback(); } console.log(outer());",
            "fallback",
        ),
    ] {
        let output = minify(source, SourceType::mjs(), true);
        assert!(output.contains(&format!("{name} =")), "{output}");
        assert!(output.contains(&format!("{name}()")), "{output}");
        let output = minify(source, SourceType::mjs(), false);
        assert!(!output.contains(name), "{output}");
    }
}

#[test]
fn annex_b_aliased_function_names_are_kept() {
    let output = minify(
        "function outer() { var value = 1; { function value() {} } return value; } use(outer());",
        SourceType::script(),
        true,
    );
    assert!(output.contains("var value = 1"), "{output}");
    assert!(output.contains("function value("), "{output}");
    assert!(output.contains("return value;"), "{output}");
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

#[test]
fn annex_b_aliases_keep_resolved_reference_membership_consistent() {
    let source = "function outer() { var value = 1; { function value() { return value; } use(value); } return value; } use(outer());";
    for keep_names in [false, true] {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::script()).parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
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
        let scoping = result.scoping.expect("mangling must return its scoping");
        let mut references = 0;
        for symbol_id in scoping.symbol_ids() {
            for &reference_id in scoping.get_resolved_reference_ids(symbol_id) {
                assert_eq!(
                    scoping.get_reference(reference_id).symbol_id(),
                    Some(symbol_id),
                    "reference {reference_id:?} belongs to {symbol_id:?}, keep_names={keep_names}"
                );
                references += 1;
            }
        }
        assert_eq!(references, 4, "every resolved reference must occur once");
    }
}

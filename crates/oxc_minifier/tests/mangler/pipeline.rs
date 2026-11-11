use std::process::Command;

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Statement};
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames};
use oxc_minifier::{CompressOptions, Compressor, Minifier, MinifierOptions};
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
    minify_with_options(
        source,
        source_type,
        MinifierOptions {
            compress,
            mangle: Some(MangleOptions {
                top_level: Some(true),
                keep_names: MangleOptionsKeepNames { function: keep_names, class: false },
                ..MangleOptions::default()
            }),
            ..MinifierOptions::default()
        },
    )
}

fn minify_with_options(source: &str, source_type: SourceType, options: MinifierOptions) -> String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    assert!(parsed.diagnostics.is_empty());
    let mut program = parsed.program;
    let result = Minifier::new(options).minify(&allocator, &mut program);
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
fn mangler_scratch_does_not_remain_in_the_ast_arena() {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        "function computeLongValue(input) { let intermediate = input * 2; return intermediate + 1; } console.log(computeLongValue(3));",
        SourceType::mjs(),
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut program = parsed.program;
    let ast_bytes = allocator.used_bytes();
    let result = Minifier::new(MinifierOptions {
        compress: None,
        mangle: Some(MangleOptions { top_level: Some(true), ..MangleOptions::default() }),
        ..MinifierOptions::default()
    })
    .minify(&allocator, &mut program);
    assert_eq!(
        allocator.used_bytes(),
        ast_bytes,
        "mangling scratch must not extend the AST arena's lifetime"
    );
    let code = Codegen::new().with_scoping(result.scoping).build(&program).code;
    let output = Command::new("node")
        .args(["--input-type=module", "--eval", &code])
        .output()
        .expect("Node.js is required to verify generated code after mangling scratch is freed");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"7\n", "{code}");
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
    for reads in [1, 1024] {
        let uses = "use(value);".repeat(reads);
        let source = format!(
            "function outer() {{ var value = 1; {{ function value() {{ return value; }} {uses} }} return value; }} use(outer());"
        );
        for keep_names in [false, true] {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &source, SourceType::script()).parse();
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

            assert_eq!(references, reads + 3, "every resolved reference must occur once");
        }
    }
}

#[test]
fn annex_b_copies_and_barriers_survive_compression_and_mangling() {
    for source in [
        "function run(){var value='outer',count=1;try{throw'catch'}catch(value){{let other=1;function value(){}console.log(other)}}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer';try{throw'catch'}catch(value){{let other=1;function value(){}console.log(other)}console.log(value)}console.log(typeof value)}run();",
        "function run(){var value='outer',count=1;try{throw'catch'}catch(value){try{throw'nested'}catch(value){{let other=1;function value(){}console.log(other)}}console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;try{throw{value:'catch'}}catch({value}){{let other=1;function value(){}console.log(other)}console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{let value=globalThis.first;globalThis.read=()=>value;value=globalThis.second;{let other=1;function value(){}console.log(other)}console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(value){var count=1;{let other=1;function value(){}console.log(other)}console.log(typeof value,typeof count)}run('outer');",
        "function run(){var count=1;{let value=1;{function value(){}}console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{let value='block';{let other=1;function value(){}console.log(other)}console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{let value='block';{function value(){}}}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{class value{};{function value(){}}}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{let value=1;if(1)function value(){};console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var count=1;{let value=1;if(1)function value(){};console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{const value=1;if(1)function value(){};console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{class value{read(){return 1}}if(1)function value(){};console.log(new value().read())}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{let value=1;if(1)function value(){}}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;{class value{};if(1)function value(){}}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;try{throw'catch'}catch(value){if(1)function value(){};console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;try{throw{value:'catch'}}catch({value}){if(1)function value(){};console.log(value)}console.log(typeof value,typeof count)}run();",
        "function run(){var value='outer',count=1;try{throw{value:'catch'}}catch({value}){if(1)function value(){}}console.log(typeof value,typeof count)}run();",
    ] {
        let execute = |code: &str| {
            let output = Command::new("node")
                .args(["--input-type=commonjs", "--eval", code])
                .output()
                .expect("Node.js is required to verify Annex B catch bindings");
            assert!(
                output.status.success(),
                "code:\n{code}\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            output.stdout
        };
        let expected = execute(source);
        for keep_names in [false, true] {
            for compress in [None, Some(CompressOptions::smallest())] {
                let minified =
                    minify_with_compression(source, SourceType::script(), keep_names, compress);
                assert_eq!(
                    execute(&minified),
                    expected,
                    "keep_names={keep_names}, source={source}\n{minified}"
                );
            }
        }
        for source_type in [SourceType::script(), SourceType::cjs()] {
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, source, source_type).parse();
            assert!(parsed.diagnostics.is_empty());
            let mut program = parsed.program;
            Compressor::new(&allocator)
                .dead_code_elimination(&mut program, CompressOptions::smallest());
            let minified = Codegen::new().build(&program).code;
            assert_eq!(execute(&minified), expected, "DCE source={source}\n{minified}");
            for mut compress in [CompressOptions::smallest(), CompressOptions::safest()] {
                compress.unused = oxc_minifier::CompressOptionsUnused::Keep;
                let minified = minify_with_options(
                    source,
                    source_type,
                    MinifierOptions {
                        compress: Some(compress),
                        mangle: None,
                        ..MinifierOptions::default()
                    },
                );
                assert_eq!(execute(&minified), expected, "source={source}\n{minified}");
            }
        }
    }
}

#[test]
fn substituted_iife_arguments_preserve_tdz_exceptions() {
    for source in [
        "function use(value) {} use((x => x)(void later)); let later = 1;",
        "function use(value) {} use((function (x) { return x; })(void later)); let later = 1;",
        "globalThis.probe = function (first = (x => x)(void later), later = 1) { return first; }; globalThis.probe();",
        "class Child extends Object { constructor() { console.log((x => x)(void this)); super(); } } new Child();",
    ] {
        let minified = minify_with_compression(
            source,
            SourceType::mjs(),
            false,
            Some(CompressOptions::smallest()),
        );
        for code in [source, &minified] {
            let output = Command::new("node")
                .args(["--input-type=module", "--eval", code])
                .output()
                .expect("Node.js is required to verify TDZ exceptions");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(!output.status.success(), "TDZ read must throw:\n{source}\noutput:\n{code}");
            assert!(stderr.contains("ReferenceError:"), "{code}\n{stderr}");
        }
    }
}

#[test]
fn unused_class_self_references_do_not_escape_the_class() {
    for source in [
        "class Dead { static self = Dead; clone() { return new Dead(); } } console.log('ok');",
        "const unused = class Dead { static self = Dead; clone() { return new Dead(); } }; console.log('ok');",
        "(class Dead { static self = Dead; }); console.log('ok');",
        "const unused = class Dead { static self = [Dead]; }; console.log('ok');",
        "const unused = class Dead { static self = (0, Dead); }; console.log('ok');",
        "const unused = class Dead { static self = !Dead; }; console.log('ok');",
        "const unused = class Dead { static self = { value: Dead }; }; console.log('ok');",
        "try { const unused = class Dead { static self = [Dead, later]; }; let later; } catch (error) { console.log(error instanceof ReferenceError, error.message.includes('before initialization')); }",
        "const Dead = 'outer'; const unused = class Dead { static self = [Dead]; }; console.log(Dead);",
        "class Live { static self = Live; } console.log(Live.self === Live);",
        "class Live { static self = Live; static { console.log(Live.self === Live); } } console.log('ok');",
        "class Dead { [Dead]() {} } console.log('ok');",
        "class Dead extends Dead {} console.log('ok');",
    ] {
        let execute = |code: &str| {
            Command::new("node")
                .args(["--input-type=module", "--eval", code])
                .output()
                .expect("Node.js is required to verify class initialization")
        };
        let reference = execute(source);
        for keep_names in [false, true] {
            let mut compress = CompressOptions::smallest();
            compress.keep_names.function = keep_names;
            compress.keep_names.class = keep_names;
            let minified =
                minify_with_compression(source, SourceType::mjs(), keep_names, Some(compress));
            let output = execute(&minified);
            assert_eq!(output.status.code(), reference.status.code(), "{source}\n{minified}");
            assert_eq!(output.stdout, reference.stdout, "{source}\n{minified}");
            if !reference.status.success() {
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains("ReferenceError:"),
                    "{source}\n{minified}\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

#[test]
fn exported_closures_preserve_tdz_errors_in_module_cycles() {
    let source = "if (0) d(); function d() {} let x; export function c() { f(); if (x) y(); else return g; }";
    let minified = minify_with_compression(
        source,
        SourceType::mjs(),
        false,
        Some(CompressOptions::smallest()),
    );
    let script = r#"
        import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
        import { tmpdir } from 'node:os';
        import { join } from 'node:path';
        import { pathToFileURL } from 'node:url';
        const dir = mkdtempSync(join(tmpdir(), 'oxc-lexical-cycle-'));
        try {
            writeFileSync(join(dir, 'a.mjs'), "import './b.mjs';\n" + process.argv[1]);
            writeFileSync(join(dir, 'b.mjs'),
                "import { c } from './a.mjs'; globalThis.f = () => {}; globalThis.g = 'ok'; " +
                "try { console.log(c()); } catch (error) { console.log(error.name); }");
            await import(pathToFileURL(join(dir, 'a.mjs')));
        } finally {
            rmSync(dir, { recursive: true });
        }
    "#;
    for code in [source, &minified] {
        let output = Command::new("node")
            .args(["--input-type=module", "--eval", script, "--", code])
            .output()
            .expect("Node.js is required to verify cyclic-module initialization");
        assert!(output.status.success(), "{code}\n{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, b"ReferenceError\n", "{code}");
    }
}

#[test]
fn property_extraction_preserves_tdz_exceptions() {
    for source in [
        "globalThis.result = (({p: void later}).p === void 0); let later = 1;",
        "globalThis.result = (({p: void later})['p'] === void 0); let later = 1;",
    ] {
        let minified = minify_with_compression(
            source,
            SourceType::mjs(),
            false,
            Some(CompressOptions::smallest()),
        );
        for code in [source, &minified] {
            let output = Command::new("node")
                .args(["--input-type=module", "--eval", code])
                .output()
                .expect("Node.js is required to verify property extraction exceptions");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(!output.status.success(), "TDZ read must throw:\n{source}\noutput:\n{code}");
            assert!(stderr.contains("ReferenceError:"), "{code}\n{stderr}");
        }
    }
}

#[test]
fn constant_extraction_preserves_shadowed_undefined_tdz() {
    for source in [
        "const f = () => void undefined; console.log(f()); let undefined;",
        "const f = unused => void undefined; console.log(f(0)); let undefined;",
        "const obj = { x: void undefined }; console.log(obj.x); let undefined;",
        "const obj = { x: +void undefined }; console.log(obj.x); let undefined;",
        "let E; (e => { e.X = +void undefined; })(E || (E = {})); console.log(E.X); let undefined;",
        "let E; (e => { e[e.X = +void undefined] = 'X'; })(E || (E = {})); console.log(E.X); let undefined;",
    ] {
        for keep_names in [false, true] {
            let minified = minify_with_compression(
                source,
                SourceType::mjs(),
                keep_names,
                Some(CompressOptions::smallest()),
            );
            for code in [source, &minified] {
                let output = Command::new("node")
                    .args(["--input-type=module", "--eval", code])
                    .output()
                    .expect("Node.js is required to verify constant extraction exceptions");
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    !output.status.success(),
                    "TDZ read must throw:\n{source}\noutput:\n{code}"
                );
                assert!(stderr.contains("ReferenceError:"), "{code}\n{stderr}");
            }
        }
    }
}

#[test]
fn inline_member_writes_preserve_cross_script_accessor_reads() {
    let script = r#"
        import { createContext, runInContext } from 'node:vm';
        const context = createContext({});
        runInContext(`
            const log = [];
            Object.defineProperty(globalThis, 'read', {
                configurable: true,
                get() { log.push('read'); return true; }
            });
            function effect() { log.push('effect'); }
        `, context);
        runInContext(process.argv[1], context);
        console.log(runInContext("JSON.stringify(log)", context));
    "#;
    for (expression, expected) in [
        ("({ x: read && 0 }).x = effect()", "[\"read\",\"effect\"]\n"),
        ("({ x: read && 0 })['x'] = effect()", "[\"read\",\"effect\"]\n"),
        ("({ x: read ? 0 : 1 }).x = effect()", "[\"read\",\"effect\"]\n"),
        ("({ x: 0 }).x = (read && 0)", "[\"read\"]\n"),
        ("({ x: read && 0 }).x = (read && 0)", "[\"read\",\"read\"]\n"),
    ] {
        let source = format!("var read; function run() {{ {expression}; }} run();");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::script()).parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let mut program = parsed.program;
        Minifier::new(MinifierOptions {
            compress: Some(CompressOptions::smallest()),
            mangle: None,
            ..MinifierOptions::default()
        })
        .minify(&allocator, &mut program);
        let minified = Codegen::new().build(&program).code;
        for code in [&source, &minified] {
            let output = Command::new("node")
                .args(["--input-type=module", "--eval", script, "--", code])
                .output()
                .expect("Node.js is required to verify cross-script accessor reads");
            assert!(output.status.success(), "{code}\n{}", String::from_utf8_lossy(&output.stderr));
            assert_eq!(output.stdout, expected.as_bytes(), "{source}\n{code}");
        }
    }
}

#[test]
fn property_extraction_preserves_inferred_stack_names() {
    for source in [
        "console.log(({ descriptive: function () { return new Error().stack; } }).descriptive());",
        "console.log(({ descriptive: () => new Error().stack })['descriptive']());",
    ] {
        for keep_names in [true, false] {
            let mut compress = CompressOptions::smallest();
            compress.keep_names.function = keep_names;
            let minified =
                minify_with_compression(source, SourceType::mjs(), keep_names, Some(compress));
            let reference_name = stack_function_name(source, SourceType::mjs());
            let minified_name = stack_function_name(&minified, SourceType::mjs());
            if keep_names {
                assert_eq!(minified_name, reference_name, "{minified}");
            } else {
                assert_ne!(minified_name, reference_name, "{minified}");
            }
        }
    }
}

#[test]
fn named_iifes_preserve_stack_names() {
    for source in [
        "(function descriptive() { console.log(new Error().stack); })();",
        "(function descriptive(value) { console.log(new Error().stack); return value; })(1);",
        "(function descriptive() { console.log(new Error().stack); console.log('tail'); return 0; })();",
    ] {
        assert_eq!(stack_function_name(source, SourceType::mjs()), "descriptive", "{source}");
        for keep_names in [true, false] {
            let mut compress = CompressOptions::smallest();
            compress.keep_names.function = keep_names;
            let minified =
                minify_with_compression(source, SourceType::mjs(), keep_names, Some(compress));
            let name = stack_function_name(&minified, SourceType::mjs());
            if keep_names {
                assert_eq!(name, "descriptive", "{minified}");
            } else {
                assert_ne!(name, "descriptive", "{minified}");
            }
        }
    }
}

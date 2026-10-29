use crate::util::SemanticTester;

#[test]
fn test_class_simple() {
    SemanticTester::js(
        "
      class Foo {
        #privateProperty = 1;
        publicProperty = 2;

        constructor() {} // this method is skip
        a() {}
        set b(v) {}
        get b() {}
      }

    ",
    )
    .has_class("Foo")
    .has_number_of_elements(5)
    .has_method("a")
    .has_property("privateProperty");
}

#[test]
fn test_class_with_ts() {
    SemanticTester::ts(
        "
      class Foo {
        accessor ap = 1;
        accessor #pap = 1;
        constructor() {} // this method is skip
      }

    ",
    )
    .has_class("Foo")
    .has_number_of_elements(2)
    .has_accessor("ap")
    .has_accessor("pap");
}

#[test]
fn test_ancestry_vs_full_nodes_parity() {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    let cases = [
        ("class yield {}", SourceType::cjs()),
        ("class eval {}", SourceType::cjs()),
        ("class C extends (yield) {}", SourceType::cjs()),
        ("class C extends yield + 1 {}", SourceType::cjs()),
        (
            "({ get [(function (x: unknown): x is string { return typeof x === 'string'; }) + '']() {} });",
            SourceType::ts(),
        ),
    ];

    for (source, source_type) in cases {
        let allocator = Allocator::default();
        let parsed_ancestry = Parser::new(&allocator, source, source_type).parse();
        let ret_ancestry = SemanticBuilder::new()
            .with_build_nodes(false)
            .with_check_syntax_error(true)
            .build(&parsed_ancestry.program);

        let parsed_full = Parser::new(&allocator, source, source_type).parse();
        let ret_full = SemanticBuilder::new()
            .with_build_nodes(true)
            .with_check_syntax_error(true)
            .build(&parsed_full.program);

        let msgs_ancestry: Vec<String> =
            ret_ancestry.diagnostics.iter().map(ToString::to_string).collect();
        let msgs_full: Vec<String> = ret_full.diagnostics.iter().map(ToString::to_string).collect();
        assert_eq!(msgs_ancestry, msgs_full, "mismatch for case: {source}");
    }
}

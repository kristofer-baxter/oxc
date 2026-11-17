use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Globals;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Disallows assigning to or mutating variables declared outside a
    /// component or hook during render; side effects must run outside of
    /// render.
    ///
    upstream = "globals",
    ///
    /// ### Why is this bad?
    ///
    /// Components must be pure so React can render them at any time and in
    /// any order. Writing to a global during render makes the output depend
    /// on how often the component has rendered, and breaks under Strict Mode
    /// and concurrent rendering.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// let someGlobal = false;
    /// function Component() {
    ///   someGlobal = true; // assignment during render
    ///   return <div>{String(someGlobal)}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import { useEffect } from 'react';
    /// let someGlobal = false;
    /// function Component() {
    ///   useEffect(() => {
    ///     someGlobal = true;
    ///   }, []);
    ///   return <div />;
    /// }
    /// ```
    Globals,
    react,
    correctness,
    recommended,
    version = "1.79.0",
    short_description = "Disallow assigning to or mutating globals during render.",
);

impl Rule for Globals {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Globals);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Globals may be reassigned in effects.
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
import {useEffect} from 'react';
let someGlobal = false;
function Component() {
  useEffect(() => {
    someGlobal = true;
  }, []);
  return <div />;
}
",
        // A `memo` wrapper bound to a non-component name does not open the
        // React Compiler gate, matching upstream.
        "
let someGlobal = false;
export const helper = memo(() => {
  someGlobal = true;
  return <div />;
});
",
    ];

    let fail = vec![
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
let someGlobal = false;
function Component() {
  const setGlobal = () => {
    someGlobal = true;
  };
  setGlobal();
  return <div>{String(someGlobal)}</div>;
}
",
        // Unlike upstream's file gate, `memo`/`forwardRef`-wrapped components are
        // compiled even when they are the only component in the file.
        "
let someGlobal = false;
export const Component = React.memo(function (props) {
  someGlobal = true;
  return <div>{props.text}</div>;
});
",
        "
let someGlobal = false;
export const Input = memo(forwardRef((props, ref) => {
  someGlobal = true;
  return <input ref={ref} />;
}));
",
        "
let someGlobal = false;
export default memo(() => {
  someGlobal = true;
  return <div />;
});
",
    ];

    Tester::new(Globals::NAME, Globals::PLUGIN, pass, fail).test_and_snapshot();
}

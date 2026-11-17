let someGlobal = false;

// Only a `React.memo` call at the top level, bound to a name that is not
// component-like, so the file is skipped like in `eslint-plugin-react-hooks`.
// Component-named `memo`/`forwardRef` bindings are covered by the
// `react/globals` rule tests.
export const helper = React.memo(function (props) {
  someGlobal = true;
  return <div>{props.text}</div>;
});

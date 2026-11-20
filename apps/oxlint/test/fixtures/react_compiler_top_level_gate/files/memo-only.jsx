let someGlobal = false;

// Only a `React.memo` call at the top level, bound to a name that is not
// component-like. The compiler still treats the callback as a component, so
// the file is linted.
export const helper = React.memo(function (props) {
  someGlobal = true;
  return <div>{props.text}</div>;
});

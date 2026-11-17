function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const a = [];
  a.safe = () => {};
  const b = [];
  b[props.k] = run;
  Object.assign(a, b);
  a.safe();
  return null;
}

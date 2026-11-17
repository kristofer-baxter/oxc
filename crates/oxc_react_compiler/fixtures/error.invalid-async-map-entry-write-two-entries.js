function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const m = new Map([
    ["x", safe],
    ["y", run],
  ]);
  const a = [...m];
  a[0][1] = safe;
  a[1][1]();
  return null;
}

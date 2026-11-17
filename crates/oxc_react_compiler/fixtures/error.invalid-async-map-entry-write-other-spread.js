function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const m = new Map([["k", run]]);
  const a = [...m];
  a[0][1] = safe;
  const b = [...m];
  b[0][1]();
  return null;
}

function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const m = new Map();
  m.set("k", run);
  m[props.k] = () => {};
  m.clear();
  m.get("k")();
  return null;
}

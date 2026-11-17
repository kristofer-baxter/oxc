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
  const source = props.c ? { get: () => safe } : null;
  Object.assign(m, source);
  m.get("k")();
  return null;
}

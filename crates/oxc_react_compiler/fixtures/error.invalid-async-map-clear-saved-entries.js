function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const m = new Map([["k", run]]);
  const s = new Set(m);
  m.clear();
  s.forEach(e => e[1]());
  return null;
}

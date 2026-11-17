function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const m1 = new Map([["k", safe]]);
  const m2 = new Map();
  const s = new Set(m1);
  (props.c ? m1 : m2).set("k", run);
  s.forEach(e => e[1]());
  return null;
}

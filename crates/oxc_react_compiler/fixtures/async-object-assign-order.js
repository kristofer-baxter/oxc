function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const o = {};
  Object.assign(o, { f: run }, { f: safe });
  o.f();
  return null;
}

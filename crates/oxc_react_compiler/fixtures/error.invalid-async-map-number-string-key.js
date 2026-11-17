function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const m = new Map();
  m.set(1, run);
  m.set("1", safe);
  m.forEach(f => f());
  return null;
}

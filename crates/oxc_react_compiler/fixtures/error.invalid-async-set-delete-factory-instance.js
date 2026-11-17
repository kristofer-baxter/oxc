function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const make = () => async () => {
    await 0;
    bad();
  };
  const s = new Set();
  s.add(make());
  const last = make();
  s.add(last);
  s.delete(last);
  s.forEach(f => f());
  return null;
}

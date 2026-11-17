function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const a = [];
  const b = [];
  b.push(run);
  Object.assign(a, b);
  a[0]();
  return null;
}

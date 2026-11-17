function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const a = [safe, run];
  a.copyWithin(0, 1);
  a[0]();
  return null;
}

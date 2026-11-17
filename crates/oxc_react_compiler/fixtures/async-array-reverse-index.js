function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const a = [run, safe];
  a.reverse();
  a[0]();
  return null;
}

function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const a = [];
  a.push(run);
  a[props.k] = safe;
  a.named();
  return null;
}

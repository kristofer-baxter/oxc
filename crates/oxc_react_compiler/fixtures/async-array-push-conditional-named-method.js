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
  if (props.c) {
    a.slice = () => {};
  }
  a.slice();
  return null;
}

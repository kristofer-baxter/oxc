function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const m = new Map();
  m.set("k", run);
  if (props.c) {
    m.get = () => safe;
  }
  m.get("k")();
  return null;
}

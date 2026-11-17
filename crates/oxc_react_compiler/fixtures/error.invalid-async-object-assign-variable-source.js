function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const o = { f: run };
  const source = props.c ? { f: safe } : undefined;
  Object.assign(o, source);
  o.f();
  return null;
}

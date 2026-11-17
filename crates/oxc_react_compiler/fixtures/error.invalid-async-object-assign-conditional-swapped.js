function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const o = { f: safe };
  Object.assign(o, props.c ? { f: safe } : { f: run });
  o.f();
  return null;
}

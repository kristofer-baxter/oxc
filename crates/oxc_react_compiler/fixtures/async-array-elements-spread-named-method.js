function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const b = [];
  b.push(run);
  const o = { safe: () => {}, ...b };
  o.safe();
  return null;
}

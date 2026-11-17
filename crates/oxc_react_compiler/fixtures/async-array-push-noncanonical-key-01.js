function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const a = [];
  a["01"] = safe;
  a.push(run);
  a.fill(run);
  a["01"]();
  return null;
}

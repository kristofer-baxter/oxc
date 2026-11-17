function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const a = [];
  a["4294967295"] = safe;
  a.unshift(run);
  a.reverse();
  a.sort();
  a["4294967295"]();
  return null;
}

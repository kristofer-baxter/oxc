function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const maps = [];
  for (let i = 0; i < 2; i++) {
    maps.push(new Map());
  }
  maps[0].set("k", run);
  maps[1].delete("k");
  maps[0].get("k")();
  return null;
}

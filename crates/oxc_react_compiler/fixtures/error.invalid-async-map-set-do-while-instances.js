function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const maps = [];
  let i = 0;
  do {
    const m = new Map();
    maps.push(m);
    i++;
  } while (i < 2);
  maps[0].set("k", run);
  maps[1].set("k", safe);
  maps[0].get("k")();
  return null;
}

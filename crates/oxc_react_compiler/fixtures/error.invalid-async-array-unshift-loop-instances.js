function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const lists = [];
  let i = 0;
  do {
    const a = [run, safe];
    lists.push(a);
    i++;
  } while (i < 2);
  lists[1].unshift(safe);
  lists[0][0]();
  return null;
}

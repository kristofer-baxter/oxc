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
    const a = [safe, run];
    lists.push(a);
    i++;
  } while (i < 2);
  lists[1].pop();
  lists[0][1]();
  return null;
}

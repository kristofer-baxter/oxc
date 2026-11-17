function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const a = [];
  a.safe = () => {};
  a.push(run);
  a.unshift(run);
  a.splice(0, 0, run);
  a.fill(run);
  a.safe();
  return null;
}

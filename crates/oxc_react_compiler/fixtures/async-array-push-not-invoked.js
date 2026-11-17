function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const a = [];
  a.push(run);
  a.push(1);
  return null;
}

function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const a = [];
  a.unshift(run);
  a[0]();
  return null;
}

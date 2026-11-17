function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const a = [];
  a.splice(0, 0, run);
  a[0]();
  return null;
}

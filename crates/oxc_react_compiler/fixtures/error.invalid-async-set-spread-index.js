function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const a = [...new Set([run])];
  a[0]();
  return null;
}

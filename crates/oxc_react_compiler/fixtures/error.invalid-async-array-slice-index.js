function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const a = [run].slice();
  a[0]();
  return null;
}

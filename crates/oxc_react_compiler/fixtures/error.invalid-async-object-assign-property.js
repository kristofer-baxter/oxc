function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const o = Object.assign({}, {run});
  o.run();
  return null;
}

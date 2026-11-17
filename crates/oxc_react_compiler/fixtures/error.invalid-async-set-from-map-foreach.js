function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
new Set(new Map([["k", run]])).forEach(e => e[1]());
  return null;
}

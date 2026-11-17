function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
new Map(new Set([["k", run]])).forEach(f => f());
  return null;
}

function Component() {
  let count = 0;
  const bad = () => count++;
  let f = bad;
  const run = async () => {
    await 0;
    f();
  };
  const m = new Map();
  m.set("x", run);
  f = () => {};
  m.set("y", () => {});
  m.forEach(g => g());
  return null;
}

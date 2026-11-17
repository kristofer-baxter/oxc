function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const m = new Map();
  m.set("k", run);
  m.delete("k");
  m.forEach(f => f());
  return null;
}

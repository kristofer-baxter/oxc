function Component() {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const s = new Set();
  s.add(run);
  s.delete(run);
  s.forEach(f => f());
  return null;
}

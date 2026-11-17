function Component({k}) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const s = new Set(); s.add(run); s.add(run.bind(null)); s.delete(run); s.forEach(f => f());
  return null;
}

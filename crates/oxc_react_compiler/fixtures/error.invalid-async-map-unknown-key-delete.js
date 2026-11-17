function Component({k}) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const m = new Map(); m.set("x", run); m.delete(k); m.get("x")();
  return null;
}

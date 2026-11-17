function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const m = new Map();
  m.set("k", run);
  const entries = [...m];
  m.clear();
  entries[0][1]();
  return null;
}

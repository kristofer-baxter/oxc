function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
const m = new Map();
  m.set(...["k", run]);
  m.get("k")();
  return null;
}

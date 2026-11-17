function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const b = [];
  b.push(run);
  function visit(...xs) {
    xs.toString();
  }
  visit(...b);
  return null;
}

function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const objects = [];
  for (let i = 0; i < 2; i++) {
    objects.push({ f: run });
  }
  objects[1].f = safe;
  objects[0].f();
  return null;
}

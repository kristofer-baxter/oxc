function Component(props) {
  let count = 0;
  const bad = () => count++;
  const run = async () => {
    await 0;
    bad();
  };
  const safe = () => {};
  const objects = [];
  let i = 0;
  do {
    objects.push({ f: run });
    i++;
  } while (i < 2);
  Object.assign(objects[1], { f: safe });
  objects[0].f();
  return null;
}

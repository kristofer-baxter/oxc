function Component() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const task = async () => {
    await 0;
    mutate();
  };
  for (const run of [task]) {
    run();
  }
  return null;
}

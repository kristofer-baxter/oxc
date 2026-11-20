function Component() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const run = async () => {
    await 0;
    mutate();
  };
  Object.values({run})[0]();
  return null;
}

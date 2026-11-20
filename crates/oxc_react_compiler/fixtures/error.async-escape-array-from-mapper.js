function Component() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const task = async () => {
    await 0;
    mutate();
  };
  Array.from([0], () => task)[0]();
  return null;
}

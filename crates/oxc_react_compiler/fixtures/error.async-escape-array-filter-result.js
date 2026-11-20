function Component() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const task = async () => {
    await 0;
    mutate();
  };
  [task].filter(() => true)[0]();
  return null;
}

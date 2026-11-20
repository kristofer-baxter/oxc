function Component() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  new Promise(async () => {
    await 0;
    mutate();
  });
  return null;
}

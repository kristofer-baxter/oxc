function Component() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const task = async () => {
    await 0;
    mutate();
  };
  const [selected] = new Set([task]);
  selected();
  return null;
}

function Component() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const task = async () => {
    await 0;
    mutate();
  };
  const wrap = t => ({task: t});
  wrap(task).task();
  return null;
}

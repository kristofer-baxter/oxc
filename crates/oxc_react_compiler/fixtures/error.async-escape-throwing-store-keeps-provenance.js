function useBox(flag) {
  let n = 0;
  const box = {
    fn: () => {
      n = 1;
    },
  };
  const task = async () => {
    await 0;
    box.fn();
  };
  task();
  try {
    (flag ? box : null).fn = () => {};
  } catch {}
  return null;
}

function useHelper() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const task = async () => {
    await 0;
    mutate();
  };
  const a = [];
  function put(target) {
    target.push(task);
  }
  put(a);
  a[0]();
  return null;
}

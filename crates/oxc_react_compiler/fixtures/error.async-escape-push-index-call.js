function useActions() {
  useState(0);
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const run = async () => {
    await 0;
    mutate();
  };
  const callbacks = [];
  callbacks.push(run);
  callbacks[0]();
  return null;
}

function useMap() {
  let n = 0;
  const mutate = () => {
    n = 1;
  };
  const m = new Map([['x', mutate]]);
  m.set('x', () => {});
  const task = async () => {
    await 0;
    m.forEach(f => f());
  };
  task();
  return null;
}

function useConsumer() {
  let n = 0;
  let cb = () => {
    n = 1;
  };
  const task = async () => {
    await 0;
    cb();
  };
  cb = () => {};
  consume(task);
  cb = () => {
    n = 1;
  };
  return null;
}

// The desugared `??=` reads and stores through two bracket accesses. An object
// key would be converted to a property key twice, so `Symbol.toPrimitive` could
// choose a different property for the store than for the read.
function Component({obj, key, value}) {
  obj[key] ??= value;
  return obj;
}

function ComponentWithObjectKey({obj, value}) {
  const key = {
    calls: 0,
    [Symbol.toPrimitive]() {
      return this.calls++ === 0 ? 'read' : 'write';
    },
  };
  obj[key] ||= value;
  return obj;
}

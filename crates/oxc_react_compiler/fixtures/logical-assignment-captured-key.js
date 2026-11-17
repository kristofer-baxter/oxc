// The key is a template literal so it is already a string: an object key
// would be converted twice by the desugared read and store, which is a Todo.
function Component({value, initial, operator}) {
  const stored = {x: null};
  const obj = {};
  let key = 'a';
  Object.defineProperty(obj, 'a', {
    get() {
      key = 'b';
      return initial;
    },
    set(value) {
      stored.x = value;
    },
  });
  let result;
  if (operator === '??=') {
    result = obj[`${key}`] ??= value;
  } else if (operator === '||=') {
    result = obj[`${key}`] ||= value;
  } else {
    result = obj[`${key}`] &&= value;
  }
  return [result, stored.x, key, obj.b];
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{value: 42, initial: null, operator: '??='}],
};

// @validateRefAccessDuringRender

import {useRef} from 'react';

// The initializer adds control flow, so the store to `current` lands in a
// later block of the fallback. It is still lazy initialization.
function Component({flag}) {
  const ref = useRef(null);
  ref.current ??= flag ? 1 : 2;
  return <Child ref={ref} />;
}

function ComponentHandwritten({flag}) {
  const ref = useRef(null);
  ref.current ?? (ref.current = flag ? 1 : 2);
  return <Child ref={ref} />;
}

function Child({ref}) {
  'use no memo';
  return ref.current;
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [{flag: true}],
};

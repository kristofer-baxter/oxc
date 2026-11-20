import {useRef} from 'react';

function Component() {
  const ref = useRef(null);
  if (ref.current != null) {
    ref.current = 1;
  } else {
    ref.current = 2;
  }
  return null;
}

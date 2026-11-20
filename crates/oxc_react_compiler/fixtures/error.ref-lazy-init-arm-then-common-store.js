import {useRef} from 'react';

function Component({flag}) {
  const ref = useRef(null);
  if (ref.current == null) {
    if (flag) {
      ref.current = 1;
    }
    ref.current = 2;
  }
  return null;
}

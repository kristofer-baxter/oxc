import {useRef} from 'react';

function Component({flag}) {
  const ref = useRef(null);
  if (ref.current != null) {
  } else if (flag) {
    ref.current = 1;
  } else {
    ref.current = 2;
  }
  return null;
}

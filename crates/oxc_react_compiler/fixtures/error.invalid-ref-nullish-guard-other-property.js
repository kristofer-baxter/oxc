// @validateRefAccessDuringRender
import {useRef} from 'react';

function Component() {
  const ref = useRef(null);
  ref.foo ?? (ref.current = 1);
  return null;
}

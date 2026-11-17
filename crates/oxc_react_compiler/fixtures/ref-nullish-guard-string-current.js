// @validateRefAccessDuringRender
import {useRef} from 'react';

function Component() {
  const ref = useRef(null);
  ref['current'] ?? (ref['current'] = 1);
  return null;
}

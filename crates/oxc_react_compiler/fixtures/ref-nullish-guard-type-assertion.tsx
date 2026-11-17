// @validateRefAccessDuringRender
import {useRef} from 'react';

function Component() {
  const ref = useRef<number | null>(null);
  (ref.current as number | null) ?? (ref.current = 1);
  return null;
}

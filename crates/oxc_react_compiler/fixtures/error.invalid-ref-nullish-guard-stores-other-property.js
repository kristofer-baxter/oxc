// @validateRefAccessDuringRender
import {useRef} from 'react';

// Only a store to `current` initializes the ref. Writing another property of
// the same ref in the fallback leaves `current` unset, so both the nullish
// read and the write are render-time ref accesses.
function Component() {
  const ref = useRef(null);
  ref.current ?? (ref.foo = 1);
  return null;
}

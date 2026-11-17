// @validateRefAccessDuringRender
import {useRef} from 'react';

// Only a nullish test on `ref.current` is lazy initialization. Testing another
// member of the ref reads it during render, even when the fallback stores
// `current`.
function Component({keyName}) {
  const ref = useRef(null);
  ref[keyName] ?? (ref.current = 1);
  return null;
}

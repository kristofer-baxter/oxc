import {useState} from 'react';

// The async generator is returned, so a caller can advance it after render
// and reassign the captured binding.
function useRunner() {
  useState(0);
  let n = 0;
  const run = async function* () {
    n = 1;
  };
  return run;
}

import {useState} from 'react';

// The async generator is not iterated during render, but it escapes through
// the returned object, so a caller can advance it after render and reassign
// the captured binding.
function useActions() {
  useState(0);
  let n = 0;
  return {
    async *run() {
      n = 1;
    },
  };
}

import { useState } from "react";

// No top-level component or hook. The compiler still finds the nested
// component, so the file is linted.
export function mountProbe(): number | null {
  let result: number | null = null;
  function Probe() {
    result = useState(1)[0];
    return null;
  }
  void Probe;
  return result;
}

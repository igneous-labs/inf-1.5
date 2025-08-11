/**
 * Primitive js operations utils
 */

import type { ReadonlyUint8Array } from "@solana/kit";

export function bytesEq(a: ReadonlyUint8Array, b: ReadonlyUint8Array): boolean {
  if (a.length !== b.length) {
    return false;
  }
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) {
      return false;
    }
  }
  return true;
}

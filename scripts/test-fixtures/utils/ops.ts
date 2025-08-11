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

export function chunkArray(array: Uint8Array, chunkSize: number): Uint8Array[] {
  if (chunkSize <= 0) {
    throw new Error("Chunk size must be greater than 0");
  }

  const result: Uint8Array[] = [];

  for (let i = 0; i < array.length; i += chunkSize) {
    result.push(array.slice(i, i + chunkSize));
  }

  return result;
}

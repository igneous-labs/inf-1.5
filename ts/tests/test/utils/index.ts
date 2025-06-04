export * from "./file";
export * from "./token";
export * from "./inf";
export * from "./rpc";
export * from "./spl";

export function mapTup<T extends readonly any[], U>(
  tuple: T,
  callback: (value: T[number], index: number) => U
): { [K in keyof T]: U } {
  return tuple.map(callback) as any;
}

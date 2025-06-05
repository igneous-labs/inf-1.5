export * from "./file";
export * from "./inf";
export * from "./rpc";
export * from "./spl";
export * from "./token";
export * from "./trade";

export function mapTup<T extends readonly any[], U>(
  tuple: T,
  callback: (value: T[number], index: number) => U
): { [K in keyof T]: U } {
  return tuple.map(callback) as any;
}

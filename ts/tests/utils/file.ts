import type { AccountInfoWithBase64EncodedData, Address } from "@solana/kit";
import { readFileSync, writeFileSync } from "fs";
import { parse, stringify } from "lossless-json";

export interface TestFixtureAcc {
  pubkey: Address;
  account: AccountInfoWithBase64EncodedData;
}

export function testFixturesAccPath(name: string): string {
  return `${import.meta.dirname}/../../../test-fixtures/${name}.json`;
}

// need to use `lossless-json` instead of JSON builtin
// to make sure integers > MAX_SAFE_INTEGER dont get corrupted to floats.
// Example: rentEpoch=u64::MAX, corruption results in value
// changing and solana-test-validator being unable to start

export function testFixturesAcc(name: string): TestFixtureAcc {
  return parse(
    readFileSync(testFixturesAccPath(name), "utf8")
  ) as TestFixtureAcc;
}

export function writeTestFixturesAcc(name: string, data: TestFixtureAcc) {
  writeFileSync(testFixturesAccPath(name), stringify(data, undefined, 2)!);
}

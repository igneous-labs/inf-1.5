import type { AccountInfoWithBase64EncodedData, Address } from "@solana/kit";
import { readFileSync } from "fs";

export interface TestFixtureAcc {
  pubkey: Address;
  account: AccountInfoWithBase64EncodedData;
}

export function testFixturesAcc(fname: string): TestFixtureAcc {
  return JSON.parse(
    readFileSync(
      `${import.meta.dirname}/../../../../test-fixtures/${fname}.json`,
      "utf8"
    )
  );
}

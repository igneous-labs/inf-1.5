/**
 * Change the last upgrade slot stored in a generic sol val calculator program's
 * state account data to 0
 */

import {
  getBase64Codec,
  getU64Encoder,
  type Base64EncodedBytes,
} from "@solana/kit";
import { testFixturesAcc, writeTestFixturesAcc } from "../utils";

const LAST_UPGRADE_SLOT_OFFSET = 32;

function main() {
  const [_node, _script, name] = process.argv;

  if (!name) {
    console.log(
      "Usage: generic-calc-last-update-slot.ts <test-fixtures-filename-without-json-ext>"
    );
    return;
  }

  const acc = testFixturesAcc(name);

  const b64codec = getBase64Codec();

  const bytes = new Uint8Array(b64codec.encode(acc.account.data[0]));
  bytes.set(getU64Encoder().encode(0), LAST_UPGRADE_SLOT_OFFSET);
  acc.account.data[0] = b64codec.decode(bytes) as Base64EncodedBytes;

  writeTestFixturesAcc(name, acc);
}

main();

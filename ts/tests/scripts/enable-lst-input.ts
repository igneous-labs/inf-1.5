/**
 * Set `is_input_disabled=0` for a specific LST on LstStateList test-fixtures account data
 */

import {
  getBase64Codec,
  getAddressEncoder,
  type Base64EncodedBytes,
  address,
} from "@solana/kit";
// cannot import any modules that import vitest stuff or else
// `Error: Vitest failed to access its internal state.`
import { testFixturesAcc, writeTestFixturesAcc } from "../utils/file";
import { bytesEq } from "../utils/ops";

const IS_INPUT_DISABLED_OFFSET = 0;
const LST_STATE_SIZE = 80;
const MINT_OFFSET = 16;

const LST_STATE_LIST_NAME = "lst-state-list";

function main() {
  const [_node, _script, mintStr] = process.argv;

  if (!mintStr) {
    console.log("Usage: enable-lst-input.ts <lst-mint>");
    return;
  }

  const mint = getAddressEncoder().encode(address(mintStr));

  const acc = testFixturesAcc(LST_STATE_LIST_NAME);

  const b64codec = getBase64Codec();

  const bytes = new Uint8Array(b64codec.encode(acc.account.data[0]));

  for (let offset = 0; offset < bytes.length; offset += LST_STATE_SIZE) {
    const lstState = bytes.subarray(offset, offset + LST_STATE_SIZE);
    const lstStateMint = lstState.subarray(MINT_OFFSET, MINT_OFFSET + 32);
    if (!bytesEq(lstStateMint, mint)) {
      continue;
    }

    lstState[IS_INPUT_DISABLED_OFFSET] = 0;

    acc.account.data[0] = b64codec.decode(bytes) as Base64EncodedBytes;
    writeTestFixturesAcc(LST_STATE_LIST_NAME, acc);

    return;
  }

  throw new Error(`mint ${mint} not on list`);
}

main();

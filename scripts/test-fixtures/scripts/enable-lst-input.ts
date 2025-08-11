/**
 * Set `is_input_disabled=0` for a specific LST on LstStateList test-fixtures account data
 */

import {
  getBase64Codec,
  getAddressEncoder,
  type Base64EncodedBytes,
  address,
} from "@solana/kit";
import {
  bytesEq,
  LST_STATE_IS_INPUT_DISABLED_OFFSET,
  LST_STATE_MINT_OFFSET,
  LST_STATE_SIZE,
  testFixturesAcc,
  writeTestFixturesAcc,
} from "../utils";

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
    const lstStateMint = lstState.subarray(
      LST_STATE_MINT_OFFSET,
      LST_STATE_MINT_OFFSET + 32
    );
    if (!bytesEq(lstStateMint, mint)) {
      continue;
    }

    lstState[LST_STATE_IS_INPUT_DISABLED_OFFSET] = 0;

    acc.account.data[0] = b64codec.decode(bytes) as Base64EncodedBytes;
    writeTestFixturesAcc(LST_STATE_LIST_NAME, acc);

    return;
  }

  throw new Error(`mint ${mint} not on list`);
}

main();

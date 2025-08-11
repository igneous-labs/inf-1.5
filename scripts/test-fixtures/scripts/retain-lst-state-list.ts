/**
 * Modifies the lst-state-list.json test-fixture account by retaining only
 * entries corresponding to mints that we have test-fixtures data of
 *
 * TODO: this might result in state inconsistency since
 * pool.total_sol_value no longer eq sum of individual sol values
 */

import {
  address,
  Base64EncodedBytes,
  getAddressEncoder,
  getBase64Codec,
} from "@solana/kit";
import {
  bytesEq,
  chunkArray,
  LST_STATE_MINT_OFFSET,
  LST_STATE_SIZE,
  testFixturesAcc,
  writeTestFixturesAcc,
} from "../utils";

const LST_STATE_LIST_NAME = "lst-state-list";

const RETAIN_MINTS = [
  "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
  "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
  "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
  "So11111111111111111111111111111111111111112",
];

function main() {
  const retainMints = RETAIN_MINTS.map((m) =>
    getAddressEncoder().encode(address(m))
  );
  const acc = testFixturesAcc(LST_STATE_LIST_NAME);

  const b64codec = getBase64Codec();

  const bytes = new Uint8Array(b64codec.encode(acc.account.data[0]));

  const lstStates = chunkArray(bytes, LST_STATE_SIZE);
  const newLstStateList = lstStates.reduce((accum, lstState) => {
    const lstStateMint = lstState.subarray(
      LST_STATE_MINT_OFFSET,
      LST_STATE_MINT_OFFSET + 32
    );
    if (retainMints.find((m) => bytesEq(m, lstStateMint))) {
      return new Uint8Array([...accum, ...lstState]);
    }
    return accum;
  }, new Uint8Array());

  if (newLstStateList.length !== RETAIN_MINTS.length * LST_STATE_SIZE) {
    throw new Error(
      `Expected newLstStateList of byte length ${
        RETAIN_MINTS.length * LST_STATE_SIZE
      }. Got ${newLstStateList.length}`
    );
  }

  acc.account.data[0] = b64codec.decode(newLstStateList) as Base64EncodedBytes;
  acc.account.space = BigInt(newLstStateList.length);

  writeTestFixturesAcc(LST_STATE_LIST_NAME, acc);
}

main();

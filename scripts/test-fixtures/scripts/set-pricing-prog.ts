/**
 * Set the pricing prog of the pool-state test fixture
 */

import {
  address,
  Base64EncodedBytes,
  getAddressEncoder,
  getBase64Decoder,
  getBase64Encoder,
} from "@solana/kit";
import { testFixturesAcc, writeTestFixturesAcc } from "../utils";

const POOL_STATE_NAME = "pool-state";
const NEW_PRICING_PROG = address("s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV");
const PRICING_PROG_OFFSET = 112;

function main() {
  const acc = testFixturesAcc(POOL_STATE_NAME);

  const bytes = new Uint8Array(getBase64Encoder().encode(acc.account.data[0]));
  bytes.set(getAddressEncoder().encode(NEW_PRICING_PROG), PRICING_PROG_OFFSET);
  acc.account.data[0] = getBase64Decoder().decode(bytes) as Base64EncodedBytes;

  writeTestFixturesAcc(POOL_STATE_NAME, acc);
}

main();

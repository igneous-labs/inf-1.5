/**
 * Generates a flatslab slab PDA account using the constants specified below
 */

import {
  address,
  Address,
  getAddressComparator,
  getAddressDecoder,
  getAddressEncoder,
  getBase64Codec,
  getBase64Decoder,
  getI32Encoder,
  lamports,
  type Base64EncodedBytes,
} from "@solana/kit";
import { testFixturesAcc, writeTestFixturesAcc } from "../utils";
import assert from "assert";

const FLATSLAB_SLAB_NAME = "flatslab-slab";

const ENTRY_LEN = 40;

type Entry = { mint: Address; inpNanos: number; outNanos: number };

// unsorted
const ENTRIES: Array<Entry> = [
  {
    mint: address("jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v"),
    inpNanos: -1_000_000,
    outNanos: -2_000_000,
  },
  {
    mint: address("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"),
    inpNanos: 3_000_000,
    outNanos: 4_000_000,
  },
  {
    mint: address("So11111111111111111111111111111111111111112"),
    inpNanos: 5_000_000,
    outNanos: 6_000_000,
  },
  {
    mint: address("7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj"),
    inpNanos: 7_000_000,
    outNanos: 8_000_000,
  },
  {
    mint: address("5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm"),
    inpNanos: 9_000_000,
    outNanos: 10_000_000,
  },
];

const EXPECTED_ACC_LEN = ENTRIES.length * ENTRY_LEN + 32;

assert(EXPECTED_ACC_LEN === 232, `Actual: ${EXPECTED_ACC_LEN}`);

function entryBytes({ mint, inpNanos, outNanos }: Entry): Uint8Array {
  const bytes = new Uint8Array(ENTRY_LEN);
  bytes.set(getAddressEncoder().encode(mint), 0);
  bytes.set(getI32Encoder().encode(inpNanos), 32);
  bytes.set(getI32Encoder().encode(outNanos), 36);
  return bytes;
}

// DO NOT USE `@solana/kit:getAddressComparator()`,
// that sorts by base58 string order.
function entryCmp(aEntry: Uint8Array, bEntry: Uint8Array): number {
  const apk = aEntry.slice(0, 32);
  const bpk = bEntry.slice(0, 32);
  for (let i = 0; i < 32; i++) {
    const a = apk[i];
    const b = bpk[i];
    if (a < b) {
      return -1;
    } else if (a > b) {
      return 1;
    }
  }
  return 0;
}

function main() {
  const entries = ENTRIES.map(entryBytes).sort(entryCmp);

  // admin is just all zeros
  const data = new Uint8Array([
    ...getAddressEncoder().encode(address("11111111111111111111111111111111")),
    ...entries.flatMap((a) => [...a]),
  ]);

  assert(data.length === EXPECTED_ACC_LEN, `Actual: ${data.length}`);

  // NB: order of the fields here affect order of the fields in json
  writeTestFixturesAcc(FLATSLAB_SLAB_NAME, {
    pubkey: address("4T9YzXnmQFMyYi2nrxyXjhtUANavmCkxGCsU3GKaNjwT"),
    account: {
      lamports: lamports(2_505_600n), // solana rent 232
      data: [getBase64Decoder().decode(data) as Base64EncodedBytes, "base64"],
      owner: address("s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV"),
      executable: false,
      rentEpoch: 18446744073709551615n,
      space: BigInt(EXPECTED_ACC_LEN),
    },
  });
}

main();

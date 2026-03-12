/**
 * Override PoolStateV2 data
 */

import {
  initSyncEmbed,
  init,
  type PoolStateV2,
  getPoolState,
  setPoolState,
  serPoolState,
} from "@sanctumso/inf1";
import { basename } from "path";
import { parse, stringify } from "lossless-json";
import type { TestFixtureAcc } from "../utils";
import {
  address,
  getBase64Decoder,
  getBase64Encoder,
  type Base64EncodedBytes,
} from "@solana/kit";

initSyncEmbed();

// duplicate: importing from ../utils results in `Error: Vitest failed to access its internal state.`
const POOL_STATE_ID = address("AYhux5gJzCoeoc1PoJ1VxwPDe22RwcvpHviLDD1oCGvW");
const LST_STATE_LIST_ID = address(
  "Gb7m4daakbVbrFLR33FKMDVMHAprRZ66CSYt4bpFwUgS",
);

const OVERRIDES: Partial<PoolStateV2> = {
  rps: 39328803111936n,
  withheldLamports: 999_999_999n,
  lastReleaseSlot: 0n,
  protocolFeeLamports: 69n,
};

function main() {
  const [_node, _script, fixture] = process.argv;

  if (!fixture) {
    console.log(
      `Usage: ${basename(import.meta.url)} "$(< <test-fixture.json>)"`,
    );
    return;
  }

  const { pubkey, account } = parse(fixture) as unknown as TestFixtureAcc;

  const decoded = {
    ...account,
    data: new Uint8Array(getBase64Encoder().encode(account.data[0])),
  };
  const inf = init(
    new Map([
      [POOL_STATE_ID as string, decoded],
      [
        LST_STATE_LIST_ID,
        {
          ...decoded,
          data: new Uint8Array(),
        },
      ],
    ]),
    new Map(),
  );

  const newPs = {
    ...getPoolState(inf),
    ...OVERRIDES,
  };

  setPoolState(inf, newPs);
  const data = serPoolState(inf);
  account.data[0] = getBase64Decoder().decode(data) as Base64EncodedBytes;
  account.space = BigInt(data.length);

  console.log(stringify({ pubkey, account }, null, 2));
}

main();

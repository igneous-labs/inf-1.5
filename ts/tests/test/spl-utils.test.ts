import {
  accountsToUpdateForTrade,
  appendSplLsts,
  hasSplData,
  Inf,
  init,
  initPks,
  initSyncEmbed,
  quoteTradeExactIn,
  tradeExactInIx,
  updateForTrade,
} from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";
import {
  fetchAccountMap,
  JUPSOL_MINT,
  LAINESOL_MINT,
  localRpc,
  WSOL_MINT,
} from "../utils";
import type { Address, Rpc, SolanaRpcApi } from "@solana/kit";

const JUPSOL_POOL = "8VpRhuxa7sUUepdY3kQiTmX9rS5vx4WgaXiAnXq4KCtr";

async function emptySplInf(rpc: Rpc<SolanaRpcApi>): Promise<Inf> {
  const pks = initPks();
  const initAccs = await fetchAccountMap(rpc, pks as Address[]);
  // init with empty SplPoolAccounts
  return init(initAccs, new Map());
}

describe("appendSplLsts test", async () => {
  beforeAll(() => initSyncEmbed());

  it("Inf able to quote for SPL LST after adding data", async () => {
    const rpc = localRpc();
    const inf = await emptySplInf(rpc);

    const mints = {
      inp: WSOL_MINT,
      out: JUPSOL_MINT,
    };
    expect(() => accountsToUpdateForTrade(inf, mints)).toThrowError();

    const newSplLsts = new Map();
    newSplLsts.set(JUPSOL_MINT, JUPSOL_POOL);
    appendSplLsts(inf, newSplLsts);

    // now stuff should work. fns below that perform full
    // update -> quote -> instruction cycle should not throw
    const updateAccs = await fetchAccountMap(
      rpc,
      accountsToUpdateForTrade(inf, mints) as Address[]
    );
    updateForTrade(inf, mints, updateAccs);
    const quote = { amt: 1_000_000_000n, mints };
    quoteTradeExactIn(inf, quote);
    tradeExactInIx(inf, {
      limit: 1_000_000_000n,
      ...quote,

      // dont care abt these pubkeys,
      // we just wanna make sure the ix function
      // doesnt throw for this test here
      signer: JUPSOL_POOL,
      tokenAccs: {
        inp: JUPSOL_POOL,
        out: JUPSOL_POOL,
      },
    });
  });

  it("hasSplData set from false to true by adding data", async () => {
    const rpc = localRpc();
    const inf = await emptySplInf(rpc);
    const mints = [JUPSOL_MINT, LAINESOL_MINT];

    expect(hasSplData(inf, mints)).toStrictEqual(
      new Uint8Array(Array.from({ length: mints.length }, () => 0))
    );

    const newSplLsts = new Map();
    newSplLsts.set(JUPSOL_MINT, JUPSOL_POOL);
    appendSplLsts(inf, newSplLsts);

    const d = hasSplData(inf, mints);
    expect(d[0]).toStrictEqual(1);
    expect(d[1]).toStrictEqual(0);
  });
});

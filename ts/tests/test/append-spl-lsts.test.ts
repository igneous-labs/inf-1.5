import {
  accountsToUpdateForTrade,
  appendSplLsts,
  init,
  initPks,
  initSyncEmbed,
  quoteTradeExactIn,
  tradeExactInIx,
  updateForTrade,
} from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";
import { fetchAccountMap, JUPSOL_MINT, localRpc, WSOL_MINT } from "../utils";

const JUPSOL_POOL = "8VpRhuxa7sUUepdY3kQiTmX9rS5vx4WgaXiAnXq4KCtr";

describe("appendSplLsts test", async () => {
  beforeAll(() => initSyncEmbed());

  it("Inf able to quote for SPL LST after adding data", async () => {
    const rpc = localRpc();
    const pks = initPks();
    const initAccs = await fetchAccountMap(rpc, pks);
    // init with empty SplPoolAccounts
    const inf = init(initAccs, new Map());
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
      accountsToUpdateForTrade(inf, mints)
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
});

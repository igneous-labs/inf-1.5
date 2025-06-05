import { describe, it } from "vitest";
import {
  INF_MINT,
  infForSwap,
  localRpc,
  mapTup,
  MSOL_MINT,
  simAssertQuoteMatchesTrade,
  testFixturesTokenAcc,
} from "../../utils";
import { quoteTradeExactIn, tradeExactInIx } from "@sanctumso/inf1";

const MINTS = { inp: MSOL_MINT, out: INF_MINT };

describe("AddLiquidity msol test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const [
      { addr: infTokenAcc },
      { addr: msolTokenAcc, owner: msolTokenAccOwner },
    ] = mapTup(["inf-token-acc", "msol-token-acc"], testFixturesTokenAcc);

    const rpc = localRpc();
    const inf = await infForSwap(rpc, MINTS);

    const quote = quoteTradeExactIn(inf, {
      amt: AMT,
      mints: MINTS,
    });
    const tradeArgs = {
      amt: AMT,
      limit: quote.out,
      mints: MINTS,
      signer: msolTokenAccOwner,
      tokenAccs: {
        inp: msolTokenAcc,
        out: infTokenAcc,
      },
    };
    const ix = tradeExactInIx(inf, tradeArgs);

    await simAssertQuoteMatchesTrade(rpc, quote, tradeArgs, ix);
  });
});

import { describe, it } from "vitest";
import {
  INF_MINT,
  infForSwap,
  localRpc,
  mapTup,
  simAssertQuoteMatchesTrade,
  STSOL_MINT,
  testFixturesTokenAcc,
} from "../../utils";
import { quoteTradeExactIn, tradeExactInIx } from "@sanctumso/inf1";

const MINTS = { inp: STSOL_MINT, out: INF_MINT };

describe("AddLiquidity msol test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const [
      { addr: infTokenAcc },
      { addr: stsolTokenAcc, owner: stsolTokenAccOwner },
    ] = mapTup(["inf-token-acc", "stsol-token-acc"], testFixturesTokenAcc);

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
      signer: stsolTokenAccOwner,
      tokenAccs: {
        inp: stsolTokenAcc,
        out: infTokenAcc,
      },
    };
    const ix = tradeExactInIx(inf, tradeArgs);

    await simAssertQuoteMatchesTrade(rpc, quote, tradeArgs, ix);
  });
});

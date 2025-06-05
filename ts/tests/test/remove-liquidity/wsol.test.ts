import { describe, it } from "vitest";
import {
  INF_MINT,
  infForSwap,
  localRpc,
  mapTup,
  simAssertQuoteMatchesTrade,
  testFixturesTokenAcc,
  WSOL_MINT,
} from "../../utils";
import { quoteTradeExactIn, tradeExactInIx } from "@sanctumso/inf1";

const MINTS = { inp: INF_MINT, out: WSOL_MINT };

describe("RemoveLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const [
      { addr: infTokenAcc, owner: infTokenAccOwner },
      { addr: wsolTokenAcc },
    ] = mapTup(["inf-token-acc", "wsol-token-acc"], testFixturesTokenAcc);

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
      signer: infTokenAccOwner,
      tokenAccs: {
        inp: infTokenAcc,
        out: wsolTokenAcc,
      },
    };
    const ix = tradeExactInIx(inf, tradeArgs);

    await simAssertQuoteMatchesTrade(rpc, quote, tradeArgs, ix);
  });
});

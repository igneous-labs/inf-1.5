import { describe, it } from "vitest";
import {
  expectInfErr,
  INF_MINT,
  infForSwap,
  localRpc,
  MSOL_MINT,
  tradeExactInBasicTest,
} from "../../utils";
import { quoteTradeExactIn } from "@sanctumso/inf1";

describe("AddLiquidity marinade test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "inf-token-acc",
    });
  });

  it("add-liquidity-fails-size-too-small", async () => {
    const rpc = localRpc();
    const mints = { inp: MSOL_MINT, out: INF_MINT };
    const inf = await infForSwap(rpc, mints);
    expectInfErr(
      () =>
        quoteTradeExactIn(inf, {
          amt: 1n,
          mints,
        }),
      "SizeTooSmallErr:trade results in zero value"
    );
  });
});

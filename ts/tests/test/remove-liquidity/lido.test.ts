import { describe, it } from "vitest";
import {
  expectInfErr,
  INF_MINT,
  infForSwap,
  localRpc,
  STSOL_MINT,
  tradeExactInBasicTest,
} from "../../utils";
import { quoteTradeExactIn } from "@sanctumso/inf1";

describe("RemoveLiquidity lido test", async () => {
  /**
   * stsol fixtures:
   * - LstStateList input_disabled reset to 0 to allow testing of RemoveLiquidity
   */
  it("fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "stsol-token-acc",
    });
  });

  it("remove-liquidity-fails-not-enough-liquidity", async () => {
    const rpc = localRpc();
    const mints = { inp: INF_MINT, out: STSOL_MINT };
    const inf = await infForSwap(rpc, mints);
    expectInfErr(
      () =>
        quoteTradeExactIn(inf, {
          amt: 1_000_000_000_000_000_000n,
          mints,
        }),
      "SizeTooLargeErr:Not enough liquidity. Tokens required: 1077459524288157339. Available: 25028"
    );
  });
});

import { describe, expect, it } from "vitest";
import {
  INF_MINT,
  infForSwap,
  localRpc,
  parseInfErr,
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
    try {
      quoteTradeExactIn(inf, {
        // a very large amount
        amt: 1_000_000_000_000_000_000n,
        mints,
      });
      expect.fail("should have thrown");
    } catch (e) {
      expect(e).toSatisfy((e) => {
        const [code] = parseInfErr(e);
        return code === "PoolErr";
      });
    }
  });
});

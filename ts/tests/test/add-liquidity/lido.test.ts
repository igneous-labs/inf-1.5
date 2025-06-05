import { describe, it } from "vitest";
import { INF_MINT, STSOL_MINT, tradeExactInBasicTest } from "../../utils";

const MINTS = { inp: STSOL_MINT, out: INF_MINT };

describe("AddLiquidity stsol test", async () => {
  /**
   * stsol fixtures:
   * - LstStateList input_disabled reset to 0 to allow testing of AddLiquidity
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "stsol-token-acc",
      out: "inf-token-acc",
    });
  });
});

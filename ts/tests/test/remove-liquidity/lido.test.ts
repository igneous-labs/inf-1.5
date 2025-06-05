import { describe, it } from "vitest";
import { INF_MINT, STSOL_MINT, tradeExactInBasicTest } from "../../utils";

const MINTS = { inp: INF_MINT, out: STSOL_MINT };

describe("AddLiquidity stsol test", async () => {
  /**
   * stsol fixtures:
   * - LstStateList input_disabled reset to 0 to allow testing of AddLiquidity
   */
  it("fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "inf-token-acc",
      out: "stsol-token-acc",
    });
  });
});

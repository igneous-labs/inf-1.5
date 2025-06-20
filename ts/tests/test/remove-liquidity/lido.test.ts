import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity lido test", async () => {
  /**
   * stsol fixtures:
   * - LstStateList input_disabled reset to 0 to allow testing of AddLiquidity
   */
  it("fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "stsol-token-acc",
    });
  });
});

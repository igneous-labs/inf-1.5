import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity lido test", async () => {
  /**
   * stsol fixtures:
   * - LstStateList input_disabled reset to 0 to allow testing of AddLiquidity
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "inf-token-acc",
    });
  });
});

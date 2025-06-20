import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity spl test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "jupsol-token-acc",
    });
  });
});

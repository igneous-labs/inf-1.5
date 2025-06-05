import { describe, it } from "vitest";
import { INF_MINT, JUPSOL_MINT, tradeExactInBasicTest } from "../../utils";

const MINTS = { inp: INF_MINT, out: JUPSOL_MINT };

describe("RemoveLiquidity spl test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "inf-token-acc",
      out: "jupsol-token-acc",
    });
  });
});

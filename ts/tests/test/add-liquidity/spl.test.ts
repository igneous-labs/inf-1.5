import { describe, it } from "vitest";
import { INF_MINT, JUPSOL_MINT, tradeExactInBasicTest } from "../../utils";

const MINTS = { inp: JUPSOL_MINT, out: INF_MINT };

describe("AddLiquidity jupsol test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "jupsol-token-acc",
      out: "inf-token-acc",
    });
  });
});

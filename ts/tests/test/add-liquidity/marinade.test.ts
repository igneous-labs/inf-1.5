import { describe, it } from "vitest";
import { INF_MINT, MSOL_MINT, tradeExactInBasicTest } from "../../utils";

const MINTS = { inp: MSOL_MINT, out: INF_MINT };

describe("AddLiquidity msol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "msol-token-acc",
      out: "inf-token-acc",
    });
  });
});

import { describe, it } from "vitest";
import { INF_MINT, MSOL_MINT, tradeExactInBasicTest } from "../../utils";

const MINTS = { inp: INF_MINT, out: MSOL_MINT };

describe("RemoveLiquidity msol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 7698n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "inf-token-acc",
      out: "msol-token-acc",
    });
  });
});

import { describe, it } from "vitest";
import { INF_MINT, tradeExactInBasicTest, WSOL_MINT } from "../../utils";

const MINTS = { inp: WSOL_MINT, out: INF_MINT };

describe("AddLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "wsol-token-acc",
      out: "inf-token-acc",
    });
  });
});

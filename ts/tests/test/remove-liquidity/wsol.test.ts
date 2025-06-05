import { describe, it } from "vitest";
import { INF_MINT, tradeExactInBasicTest, WSOL_MINT } from "../../utils";

const MINTS = { inp: INF_MINT, out: WSOL_MINT };

describe("RemoveLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, MINTS, {
      inp: "inf-token-acc",
      out: "wsol-token-acc",
    });
  });
});

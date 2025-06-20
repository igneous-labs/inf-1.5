import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "wsol-token-acc",
    });
  });
});

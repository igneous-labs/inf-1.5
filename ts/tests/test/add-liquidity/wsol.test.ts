import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "inf-token-acc",
    });
  });
});

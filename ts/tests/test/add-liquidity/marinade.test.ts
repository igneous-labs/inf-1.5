import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity marinade test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "inf-token-acc",
    });
  });
});

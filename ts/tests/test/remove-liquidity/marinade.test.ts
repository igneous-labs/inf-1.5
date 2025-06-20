import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity marinade test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 7698n;
    await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "msol-token-acc",
    });
  });
});

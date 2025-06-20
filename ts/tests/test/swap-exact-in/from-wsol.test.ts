import { describe, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("SwapExactIn from wsol test", async () => {
  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "jupsol-token-acc",
    });
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "msol-token-acc",
    });
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "stsol-token-acc",
    });
  });
});

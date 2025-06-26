import { describe, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from wsol test", async () => {
  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "jupsol-token-acc",
    });
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    await tradeExactOutBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "msol-token-acc",
    });
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactOutBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "stsol-token-acc",
    });
  });
});

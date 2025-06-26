import { describe, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from lido test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "wsol-token-acc",
    });
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "jupsol-token-acc",
    });
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactOutBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "msol-token-acc",
    });
  });
});

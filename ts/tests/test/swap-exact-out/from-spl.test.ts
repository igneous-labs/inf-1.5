import { describe, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from spl test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "wsol-token-acc",
    });
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    await tradeExactOutBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "msol-token-acc",
    });
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactOutBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "stsol-token-acc",
    });
  });
});

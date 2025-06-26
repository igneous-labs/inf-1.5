import { describe, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from marinade test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "wsol-token-acc",
    });
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "jupsol-token-acc",
    });
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactOutBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "stsol-token-acc",
    });
  });
});

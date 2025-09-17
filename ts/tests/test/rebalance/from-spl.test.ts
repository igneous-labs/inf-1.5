import { describe, it } from "vitest";
import { rebalanceBasicTest } from "../../utils";

describe("Rebalance from spl test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await rebalanceBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "wsol-token-acc",
    });
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    await rebalanceBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "msol-token-acc",
    });
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await rebalanceBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "stsol-token-acc",
    });
  });
});

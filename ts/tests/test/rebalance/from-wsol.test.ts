import { describe, it } from "vitest";
import { rebalanceBasicTest } from "../../utils";

describe("Rebalance from wsol test", async () => {
  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await rebalanceBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "jupsol-token-acc",
    });
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    await rebalanceBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "msol-token-acc",
    });
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await rebalanceBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "stsol-token-acc",
    });
  });
});

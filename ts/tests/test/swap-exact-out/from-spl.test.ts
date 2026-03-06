import { describe, expect, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from spl test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 5025126n,
        "inp": 902738816n,
        "inpSolVal": 1005025126n,
        "mints": {
          "inp": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 1000000000n,
      }
    `);
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "msol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 31n,
        "inp": 9003n,
        "inpSolVal": 10019n,
        "mints": {
          "inp": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
        "out": 7698n,
      }
    `);
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "stsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 60n,
        "inp": 7646n,
        "inpSolVal": 8509n,
        "mints": {
          "inp": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
          "out": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        },
        "out": 6969n,
      }
    `);
  });
});

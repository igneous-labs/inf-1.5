import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("SwapExactIn from wsol test", async () => {
  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 3000000n,
        "inp": 1000000000n,
        "inpSolVal": 1000000000n,
        "mints": {
          "inp": "So11111111111111111111111111111111111111112",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 895530443n,
      }
    `);
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "msol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 70n,
        "inp": 7698n,
        "inpSolVal": 7698n,
        "mints": {
          "inp": "So11111111111111111111111111111111111111112",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
        "out": 5878n,
      }
    `);
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "stsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 91n,
        "inp": 6969n,
        "inpSolVal": 6969n,
        "mints": {
          "inp": "So11111111111111111111111111111111111111112",
          "out": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        },
        "out": 5673n,
      }
    `);
  });
});

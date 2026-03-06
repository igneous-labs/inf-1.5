import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("SwapExactIn from lido test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 15761273n,
        "inp": 1000000000n,
        "inpSolVal": 1212405583n,
        "mints": {
          "inp": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 1196644310n,
      }
    `);
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 6062028n,
        "inp": 1000000000n,
        "inpSolVal": 1212405583n,
        "mints": {
          "inp": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 1083568083n,
      }
    `);
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "msol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 93n,
        "inp": 6969n,
        "inpSolVal": 8449n,
        "mints": {
          "inp": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
        "out": 6439n,
      }
    `);
  });
});

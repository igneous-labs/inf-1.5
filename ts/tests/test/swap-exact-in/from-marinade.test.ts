import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("SwapExactIn from marinade test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 11676923n,
        "inp": 1000000000n,
        "inpSolVal": 1297435839n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 1285758916n,
      }
    `);
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 1297436n,
        "inp": 1000000000n,
        "inpSolVal": 1297435839n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 1164224071n,
      }
    `);
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "stsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 100n,
        "inp": 6969n,
        "inpSolVal": 9041n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        },
        "out": 7374n,
      }
    `);
  });
});

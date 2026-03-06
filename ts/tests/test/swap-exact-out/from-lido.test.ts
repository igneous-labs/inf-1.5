import { describe, expect, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from lido test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 13171226n,
        "inp": 835670209n,
        "inpSolVal": 1013171226n,
        "mints": {
          "inp": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 1000000000n,
      }
    `);
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 5594506n,
        "inp": 922876943n,
        "inpSolVal": 1118901157n,
        "mints": {
          "inp": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 1000000000n,
      }
    `);
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "msol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 101n,
        "inp": 7542n,
        "inpSolVal": 9142n,
        "mints": {
          "inp": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
        "out": 6969n,
      }
    `);
  });
});

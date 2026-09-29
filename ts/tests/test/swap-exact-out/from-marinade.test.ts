import { describe, expect, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from marinade test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 9081736n,
        "inp": 777750782n,
        "inpSolVal": 1009081736n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 1000000000n,
      }
    `);
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 1114422n,
        "inp": 858941183n,
        "inpSolVal": 1114421073n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 1000000000n,
      }
    `);
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "stsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 94n,
        "inp": 6586n,
        "inpSolVal": 8543n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        },
        "out": 6969n,
      }
    `);
  });

  it("to sols fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "swsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 15228427n,
        "inp": 782488350n,
        "inpSolVal": 1015228427n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
        },
        "out": 1000000000n,
      }
    `);
  });
});

import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("SwapExactIn from sols test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 17000000n,
        "inp": 1000000000n,
        "inpSolVal": 1000000000n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 983000000n,
      }
    `);
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "msol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 116n,
        "inp": 7698n,
        "inpSolVal": 7698n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
        "out": 5842n,
      }
    `);
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "stsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 133n,
        "inp": 6969n,
        "inpSolVal": 6969n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        },
        "out": 5638n,
      }
    `);
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 9000000n,
        "inp": 1000000000n,
        "inpSolVal": 1000000000n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 890141092n,
      }
    `);
  });
});

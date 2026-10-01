import { describe, expect, it } from "vitest";
import { tradeExactOutBasicTest } from "../../utils";

describe("SwapExactOut from sols test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 17293998n,
        "inp": 1017293998n,
        "inpSolVal": 1017293998n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 1000000000n,
      }
    `);
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "msol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 153n,
        "inp": 10141n,
        "inpSolVal": 10141n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
        "out": 7698n,
      }
    `);
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "stsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 164n,
        "inp": 8613n,
        "inpSolVal": 8613n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        },
        "out": 6969n,
      }
    `);
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactOutBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 10110757n,
        "inp": 1123417408n,
        "inpSolVal": 1123417408n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 1000000000n,
      }
    `);
  });
});

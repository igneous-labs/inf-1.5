import { describe, expect, it } from "vitest";
import { expectLiqQuote, tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const EXPECTED_OUT = 2195356048n;

    const {
      // sol val of inp INF is variable depending on slots elapsed
      inpSolVal: _,
      out,
      ...rest
    } = await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "wsol-token-acc",
    });
    expect(rest).toMatchInlineSnapshot(`
      {
        "fee": 33431819n,
        "inp": 1000000000n,
        "mints": {
          "inp": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
          "out": "So11111111111111111111111111111111111111112",
        },
      }
    `);
    expectLiqQuote({ out, dir: "ExactIn", liq: "rem" }, EXPECTED_OUT);
  });
});

import { describe, expect, it } from "vitest";
import { expectLiqQuote, tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const EXPECTED_OUT = 441944258n;

    const { out, ...rest } = await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "inf-token-acc",
    });
    expect(rest).toMatchInlineSnapshot(`
      {
        "fee": 15000000n,
        "inp": 1000000000n,
        "inpSolVal": 1000000000n,
        "mints": {
          "inp": "So11111111111111111111111111111111111111112",
          "out": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        },
      }
    `);
    expectLiqQuote({ out, dir: "ExactIn", liq: "add" }, EXPECTED_OUT);
  });
});

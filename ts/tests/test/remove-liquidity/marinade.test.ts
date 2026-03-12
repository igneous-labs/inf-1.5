import { describe, expect, it } from "vitest";
import { expectLiqQuote, tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity marinade test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 369n;
    const EXPECTED_OUT = 623n;


    const { out, ...rest } = await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "msol-token-acc",
    });
    expect(rest).toMatchInlineSnapshot(`
      {
        "fee": 11n,
        "inp": 369n,
        "inpSolVal": 822n,
        "mints": {
          "inp": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
      }
    `);
    expectLiqQuote({ out, dir: "ExactIn", liq: "rem" }, EXPECTED_OUT,);
  });
});

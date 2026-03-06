import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity marinade test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 369n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "msol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 11n,
        "inp": 369n,
        "inpSolVal": 822n,
        "mints": {
          "inp": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
          "out": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        },
        "out": 623n,
      }
    `);
  });
});

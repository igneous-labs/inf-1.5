import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity lido test", async () => {
  /**
   * stsol fixtures:
   * - LstStateList input_disabled reset to 0 to allow testing of AddLiquidity
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "stsol-token-acc",
      out: "inf-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 20610895n,
        "inp": 1000000000n,
        "inpSolVal": 1212405583n,
        "mints": {
          "inp": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "out": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        },
        "out": 534727735n,
      }
    `);
  });
});

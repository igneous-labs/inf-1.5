import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "wsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 33431819n,
        "inp": 1000000000n,
        "inpSolVal": 2228787867n,
        "mints": {
          "inp": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
          "out": "So11111111111111111111111111111111111111112",
        },
        "out": 2195356048n,
      }
    `);
  });
});

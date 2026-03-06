import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity wsol test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "wsol-token-acc",
      out: "inf-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 15000000n,
        "inp": 1000000000n,
        "inpSolVal": 1000000000n,
        "mints": {
          "inp": "So11111111111111111111111111111111111111112",
          "out": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        },
        "out": 441944258n,
      }
    `);
  });
});

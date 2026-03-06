import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("RemoveLiquidity spl test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "jupsol-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 15601516n,
        "inp": 1000000000n,
        "inpSolVal": 2228787867n,
        "mints": {
          "inp": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
          "out": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        },
        "out": 1987939573n,
      }
    `);
  });
});

import { describe, expect, it } from "vitest";
import { tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity spl test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const quote = await tradeExactInBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "inf-token-acc",
    });
    expect(quote).toMatchInlineSnapshot(`
      {
        "fee": 10019760n,
        "inp": 1000000000n,
        "inpSolVal": 1113306651n,
        "mints": {
          "inp": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
          "out": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        },
        "out": 495016555n,
      }
    `);
  });
});

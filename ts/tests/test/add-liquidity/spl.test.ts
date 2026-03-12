import { describe, expect, it } from "vitest";
import { expectLiqQuote, tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity spl test", async () => {
  /**
   * jupsol fixtures:
   * - pool cloned from mainnet in epoch 797 with data edited to change last_update_epoch to 0
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const EXPECTED_OUT = 495016555n;

    const { out, ...rest } = await tradeExactInBasicTest(AMT, {
      inp: "jupsol-token-acc",
      out: "inf-token-acc",
    });
    expect(rest).toMatchInlineSnapshot(`
      {
        "fee": 10019760n,
        "inp": 1000000000n,
        "inpSolVal": 1113306651n,
        "mints": {
          "inp": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
          "out": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        },
      }
    `);
    expectLiqQuote({ out, dir: "ExactIn", liq: "add" }, EXPECTED_OUT,);
  });
});

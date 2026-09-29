import { describe, expect, it } from "vitest";
import { expectLiqQuote, tradeExactInBasicTest } from "../../utils";

describe("AddLiquidity sols test", async () => {
  /**
   * sols fixtures:
   * - `swsol-mint` and `swsol-pool` cloned from mainnet
   * - `swsol-token-acc`, `swsol-reserves`, `swsol-pf-accum` and
   *   `sols-calc-state` are synthesized (swsol is not yet listed on INF on
   *   mainnet, so the INF pool reserves/pf accounts dont exist yet)
   * - swsol entry appended to `lst-state-list` with
   *   `sol_value_calculator=sssQe...` (the sols sol value calculator)
   * - the sols sol value calculator program and its pool program (`so1f...`)
   *   are loaded in via `--upgradeable-program`
   */
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const EXPECTED_OUT = 420390373n;

    const { out, ...rest } = await tradeExactInBasicTest(AMT, {
      inp: "swsol-token-acc",
      out: "inf-token-acc",
    });
    expect(rest).toMatchInlineSnapshot(`
      {
        "fee": 21000000n,
        "inp": 1000000000n,
        "inpSolVal": 1000000000n,
        "mints": {
          "inp": "swso1x7A8Dy36znxtcstSVLNseeCQzNV3wVAfa5GGLu",
          "out": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        },
      }
    `);
    expectLiqQuote({ out, dir: "ExactIn", liq: "add" }, EXPECTED_OUT);
  });
});

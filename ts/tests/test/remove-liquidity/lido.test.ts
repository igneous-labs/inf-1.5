import { describe, expect, it } from "vitest";
import {
  expectInfErr,
  expectLiqQuote,
  INF_MINT,
  infForSwap,
  localRpc,
  STSOL_MINT,
  tradeExactInBasicTest,
} from "../../utils";
import { quoteTradeExactIn } from "@sanctumso/inf1";

describe("RemoveLiquidity lido test", async () => {
  /**
   * stsol fixtures:
   * - LstStateList input_disabled reset to 0 to allow testing of RemoveLiquidity
   */
  it("fixtures-basic", async () => {
    const AMT = 6969n;
    const EXPECTED_OUT = 12592n;

    const { out, ...rest } = await tradeExactInBasicTest(AMT, {
      inp: "inf-token-acc",
      out: "stsol-token-acc",
    });
    expect(rest).toMatchInlineSnapshot(`
      {
        "fee": 265n,
        "inp": 6969n,
        "inpSolVal": 15532n,
        "mints": {
          "inp": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
          "out": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        },
      }
    `);
    expectLiqQuote({ out, dir: "ExactIn", liq: "rem" }, EXPECTED_OUT);
  });

  it("remove-liquidity-fails-not-enough-liquidity", async () => {
    const rpc = localRpc();
    const mints = { inp: INF_MINT, out: STSOL_MINT };
    const inf = await infForSwap(rpc, mints);
    const err = await expectInfErr(() =>
      quoteTradeExactIn(inf, {
        amt: 1_000_000_000_000_000_000n,
        mints,
        slotLookahead: 0n,
      }),
    );
    expect(err).toMatchInlineSnapshot(
      `[Error: SizeTooLargeErr:Not enough liquidity. Tokens required: 1807067290275056190. Available: 25028]`,
    );
  });
});

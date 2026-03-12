import { describe, expect, it } from "vitest";
import {
  expectInfErr,
  expectLiqQuote,
  INF_MINT,
  infForSwap,
  localRpc,
  MSOL_MINT,
  tradeExactInBasicTest,
} from "../../utils";
import { quoteTradeExactIn } from "@sanctumso/inf1";

describe("AddLiquidity marinade test", async () => {
  it("fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    const EXPECTED_OUT = 574558571n;

    const { out, ...rest } = await tradeExactInBasicTest(AMT, {
      inp: "msol-token-acc",
      out: "inf-token-acc",
    });
    expect(rest).toMatchInlineSnapshot(`
      {
        "fee": 16866666n,
        "inp": 1000000000n,
        "inpSolVal": 1297435839n,
        "mints": {
          "inp": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "out": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        },
      }
    `);
    expectLiqQuote({ out, dir: "ExactIn", liq: "add" }, EXPECTED_OUT);
  });

  it("add-liquidity-fails-size-too-small", async () => {
    const rpc = localRpc();
    const mints = { inp: MSOL_MINT, out: INF_MINT };
    const inf = await infForSwap(rpc, mints);
    const err = await expectInfErr(() =>
      quoteTradeExactIn(inf, {
        amt: 1n,
        mints,
        slotLookahead: 0n,
      }),
    );
    expect(err).toMatchInlineSnapshot(
      `[Error: SizeTooSmallErr:trade results in zero value]`,
    );
  });
});

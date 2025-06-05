import { describe, it } from "vitest";
import {
  JUPSOL_MINT,
  MSOL_MINT,
  STSOL_MINT,
  tradeExactInBasicTest,
  WSOL_MINT,
} from "../../utils";

describe("SwapExactIn from marinade test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(
      AMT,
      { out: WSOL_MINT, inp: MSOL_MINT },
      {
        inp: "msol-token-acc",
        out: "wsol-token-acc",
      }
    );
  });

  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(
      AMT,
      { out: JUPSOL_MINT, inp: MSOL_MINT },
      {
        inp: "msol-token-acc",
        out: "jupsol-token-acc",
      }
    );
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactInBasicTest(
      AMT,
      { out: STSOL_MINT, inp: MSOL_MINT },
      {
        inp: "msol-token-acc",
        out: "stsol-token-acc",
      }
    );
  });
});

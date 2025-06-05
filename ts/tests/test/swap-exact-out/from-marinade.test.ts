import { describe, it } from "vitest";
import {
  JUPSOL_MINT,
  MSOL_MINT,
  STSOL_MINT,
  tradeExactOutBasicTest,
  WSOL_MINT,
} from "../../utils";

describe("SwapExactOut from marinade test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(
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
    await tradeExactOutBasicTest(
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
    await tradeExactOutBasicTest(
      AMT,
      { out: STSOL_MINT, inp: MSOL_MINT },
      {
        inp: "msol-token-acc",
        out: "stsol-token-acc",
      }
    );
  });
});

import { describe, it } from "vitest";
import {
  JUPSOL_MINT,
  MSOL_MINT,
  STSOL_MINT,
  tradeExactOutBasicTest,
  WSOL_MINT,
} from "../../utils";

describe("SwapExactOut from wsol test", async () => {
  it("to jupsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactOutBasicTest(
      AMT,
      { out: JUPSOL_MINT, inp: WSOL_MINT },
      {
        inp: "wsol-token-acc",
        out: "jupsol-token-acc",
      }
    );
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    await tradeExactOutBasicTest(
      AMT,
      { out: MSOL_MINT, inp: WSOL_MINT },
      {
        inp: "wsol-token-acc",
        out: "msol-token-acc",
      }
    );
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactOutBasicTest(
      AMT,
      { out: STSOL_MINT, inp: WSOL_MINT },
      {
        inp: "wsol-token-acc",
        out: "stsol-token-acc",
      }
    );
  });
});

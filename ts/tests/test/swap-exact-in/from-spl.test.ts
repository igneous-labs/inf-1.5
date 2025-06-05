import { describe, it } from "vitest";
import {
  JUPSOL_MINT,
  MSOL_MINT,
  STSOL_MINT,
  tradeExactInBasicTest,
  WSOL_MINT,
} from "../../utils";

describe("SwapExactIn from spl test", async () => {
  it("to wsol fixtures-basic", async () => {
    const AMT = 1_000_000_000n;
    await tradeExactInBasicTest(
      AMT,
      { out: WSOL_MINT, inp: JUPSOL_MINT },
      {
        inp: "jupsol-token-acc",
        out: "wsol-token-acc",
      }
    );
  });

  it("to msol fixtures-basic", async () => {
    const AMT = 7698n;
    await tradeExactInBasicTest(
      AMT,
      { out: MSOL_MINT, inp: JUPSOL_MINT },
      {
        inp: "jupsol-token-acc",
        out: "msol-token-acc",
      }
    );
  });

  it("to stsol fixtures-basic", async () => {
    const AMT = 6969n;
    await tradeExactInBasicTest(
      AMT,
      { out: STSOL_MINT, inp: JUPSOL_MINT },
      {
        inp: "jupsol-token-acc",
        out: "stsol-token-acc",
      }
    );
  });
});

import { beforeAll, describe, expect, it } from "vitest";
import { initSyncEmbed, withdrawProtocolFeesV2IxRaw } from "@sanctumso/inf1";
import { INF_MINT, POOL_STATE_ID } from "../utils";

describe("Withdraw protocol fees v2", async () => {
  // taken from accounts.test
  const PROTOCOL_FEE_BENEFICIARY = "EeQmNqm1RcQnee8LTyx6ccVG9FnR8TezQuw2JXq2LC1T";
  // inf-token-acc
  const WITHDRAW_TO = "GwEVBmBh5nvVJuH122RCrPKQGKJ6JRYz2AVvBk9tHEvo";
  // token program
  const TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
  // INF program
  const INF_PROGRAM_ID = "5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx";

  beforeAll(() => initSyncEmbed());

  it("builds withdrawProtocolFeesV2 correctly", async () => {
    const ix = withdrawProtocolFeesV2IxRaw({
      protocolFeeBeneficiary: PROTOCOL_FEE_BENEFICIARY,
      withdrawTo: WITHDRAW_TO,
      infMint: INF_MINT,
      tokenProgram: TOKEN_PROGRAM_ID,
    });

    expect(ix.programAddress).toStrictEqual(INF_PROGRAM_ID);
    expect(Array.from(ix.data)).toStrictEqual([25]);
    expect(ix.accounts.map((account) => account.address)).toStrictEqual([
      POOL_STATE_ID,
      PROTOCOL_FEE_BENEFICIARY,
      WITHDRAW_TO,
      INF_MINT,
      TOKEN_PROGRAM_ID,
    ]);
    expect(ix.accounts.map((a) => a.role)).toStrictEqual([1, 2, 1, 1, 0]);
  });
});

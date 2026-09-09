import { AccountRole, address } from "@solana/kit";
import { initSyncEmbed, updateLastUpgradeSlotIx } from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";

const MANAGER = "CK9cEJT7K7oRrMCcEbBQRGqHLGpxKXWnKvW7nHSDMHD1";
const SVC_PROGRAM = "sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF";
const SVC_STATE = "7orJ4kDhn1Ewp54j29tBzUWDFGhyimhYi7sxybZcphHd";
const POOL_PROGRAM = "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy";
const POOL_PROGRAM_DATA = "EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3";

describe("generic SVC UpdateLastUpgradeSlot instruction", () => {
  beforeAll(() => initSyncEmbed());

  it("derives the accounts and encodes the canonical ABI", () => {
    const instruction = updateLastUpgradeSlotIx({
      manager: MANAGER,
      svcProgram: SVC_PROGRAM,
      poolProgram: POOL_PROGRAM,
    });

    expect(instruction.programAddress).toBe(SVC_PROGRAM);
    expect(instruction.data).toEqual(Uint8Array.of(253));
    expect(instruction.accounts.map(({ address: accountAddress, role }) => ({
      address: address(accountAddress),
      role,
    }))).toEqual([
      { address: address(MANAGER), role: AccountRole.READONLY_SIGNER },
      { address: address(SVC_STATE), role: AccountRole.WRITABLE },
      { address: address(POOL_PROGRAM), role: AccountRole.READONLY },
      { address: address(POOL_PROGRAM_DATA), role: AccountRole.READONLY },
    ]);
  });
});

import { beforeAll, describe, expect, it } from "vitest";
import { withdrawProtocolFeesV2BasicTest } from "../../utils";
import { initSyncEmbed } from "@sanctumso/inf1";

describe("Withdraw protocol fees v2", async () => {
  beforeAll(() => {
    initSyncEmbed();
  });

  it("withdraws protocol fees to withdrawTo", async () => {
    const { poolStateBefore, poolStateAfter, infMinted, infWithdrawn } =
      await withdrawProtocolFeesV2BasicTest("inf-token-acc");

    expect(poolStateBefore.protocolFeeLamports).toBeGreaterThan(0n);
    expect(poolStateAfter.protocolFeeLamports).toEqual(0n);
    expect(infWithdrawn).toBeGreaterThan(0n);
    expect(infWithdrawn).toEqual(infMinted);
  });
});

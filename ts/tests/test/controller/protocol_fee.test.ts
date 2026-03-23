import { beforeAll, describe, expect, it } from "vitest";
import { withdrawProtocolFeesV2BasicTest } from "../../utils";
import { initSyncEmbed } from "@sanctumso/inf1";

describe("Withdraw protocol fees v2", async () => {
  it("withdraws protocol fees to withdrawTo", async () => {
    await withdrawProtocolFeesV2BasicTest("inf-token-acc");
  });
});

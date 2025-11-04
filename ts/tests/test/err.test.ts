import { allInfErrs, initSyncEmbed } from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";

describe("infErrs test", () => {
  beforeAll(() => initSyncEmbed());

  it("allInfErrs snapshot", () => {
    expect(allInfErrs()).toMatchInlineSnapshot(`
      [
        "AccDeserErr",
        "InternalErr",
        "MissingAccErr",
        "MissingSplDataErr",
        "MissingSvcDataErr",
        "NoValidPdaErr",
        "PoolErr",
        "UnknownPpErr",
        "UnknownSvcErr",
        "UnsupportedMintErr",
        "SizeTooSmallErr",
        "SizeTooLargeErr",
      ]
    `);
  });
});

import {
  deserLstStateList,
  deserPoolState,
  initPks,
  initSyncEmbed,
} from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";
import {
  fetchAccountMap,
  JUPSOL_MINT,
  localRpc,
  MSOL_MINT,
  STSOL_MINT,
  WSOL_MINT,
} from "../utils";
import { type Address, isAddress } from "@solana/kit";

describe("accounts test", () => {
  beforeAll(() => initSyncEmbed());

  it("happy path deserPoolState", async () => {
    const rpc = localRpc();
    const [poolStateAddr] = initPks() as Address[];
    const accounts = await fetchAccountMap(rpc, [poolStateAddr]);

    const poolAccount = accounts.get(poolStateAddr);
    expect(poolAccount).toBeDefined();

    const pool = deserPoolState(poolAccount!.data);

    expect(pool.totalSolValue).toBeGreaterThan(0n);
    expect(pool.tradingProtocolFeeBps).toBeGreaterThanOrEqual(0);
    expect(pool.lpProtocolFeeBps).toBeGreaterThanOrEqual(0);
    expect(pool.version).toBeGreaterThanOrEqual(0);
    expect(pool.isDisabled).toBeGreaterThanOrEqual(0);
    expect(pool.isRebalancing).toBeGreaterThanOrEqual(0);
    expect(isAddress(pool.admin)).toBe(true);
    expect(isAddress(pool.rebalanceAuthority)).toBe(true);
    expect(isAddress(pool.protocolFeeBeneficiary)).toBe(true);
    expect(isAddress(pool.pricingProgram)).toBe(true);
    expect(isAddress(pool.lpTokenMint)).toBe(true);
  });

  it("happy path deserLstStateList", async () => {
    const rpc = localRpc();
    const [, lstStateListAddr] = initPks() as Address[];
    const accounts = await fetchAccountMap(rpc, [lstStateListAddr]);

    const lstStateListAccount = accounts.get(lstStateListAddr);
    expect(lstStateListAccount).toBeDefined();

    const lstStateList = deserLstStateList(lstStateListAccount!.data);
    expect(lstStateList.states.length).toBeGreaterThan(0);

    for (const state of lstStateList.states) {
      expect(state.isInputDisabled).toBeGreaterThanOrEqual(0);
      expect(state.poolReservesBump).toBeGreaterThan(0n);
      expect(state.protocolFeeAccumulatorBump).toBeGreaterThan(0n);
      expect(state.solValue).toBeGreaterThan(0n);
      expect(isAddress(state.mint)).toBe(true);
      expect(isAddress(state.solValueCalculator)).toBe(true);
    }

    const mints = lstStateList.states.map((s) => s.mint);

    expect(mints).toContain(WSOL_MINT);
    expect(mints).toContain(STSOL_MINT);
    expect(mints).toContain(MSOL_MINT);
    expect(mints).toContain(JUPSOL_MINT);
  });
});

import {
  getPoolState,
  getLstStateList,
  init,
  initPks,
  initSyncEmbed,
  Inf,
} from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";
import {
  fetchAccountMap,
  JUPSOL_MINT,
  localRpc,
  MSOL_MINT,
  SPL_POOL_ACCOUNTS,
  STSOL_MINT,
  WSOL_MINT,
} from "../utils";
import {
  type Address,
  isAddress,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";

async function splInf(rpc: Rpc<SolanaRpcApi>): Promise<Inf> {
  const pks = initPks();
  const initAccs = await fetchAccountMap(rpc, pks as Address[]);
  // init with SPL_POOL_ACCOUNTS
  return init(initAccs, SPL_POOL_ACCOUNTS);
}

describe("accounts test", () => {
  beforeAll(() => initSyncEmbed());

  const rpc = localRpc();

  it("happy path getPoolState", async () => {
    const inf = await splInf(rpc);
    const pool = getPoolState(inf);

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

  it("happy path getLstStateList", async () => {
    const inf = await splInf(rpc);
    const lstStateList = getLstStateList(inf);
    expect(lstStateList.states.length).toBeGreaterThan(0);

    for (const state of lstStateList.states) {
      expect(state.isInputDisabled).toBeGreaterThanOrEqual(0);
      expect(state.poolReservesBump).toBeGreaterThanOrEqual(0);
      expect(state.protocolFeeAccumulatorBump).toBeGreaterThanOrEqual(0);
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

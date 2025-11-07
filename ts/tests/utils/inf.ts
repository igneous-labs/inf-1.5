import {
  accountsToUpdateForRebalance,
  accountsToUpdateForTrade,
  init,
  initPks,
  initSyncEmbed,
  updateForRebalance,
  updateForTrade,
  type Inf,
  type InfErrMsg,
  type PkPair,
} from "@sanctumso/inf1";
import {
  address,
  type Address,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { fetchAccountMap } from "./rpc";
import { SPL_POOL_ACCOUNTS } from "./spl";
import { expect } from "vitest";

export const POOL_STATE_ID = address(
  "AYhux5gJzCoeoc1PoJ1VxwPDe22RwcvpHviLDD1oCGvW"
);
export const LST_STATE_LIST_ID = address(
  "Gb7m4daakbVbrFLR33FKMDVMHAprRZ66CSYt4bpFwUgS"
);

/**
 * Initializes, updates and returns an `Inf` that is ready for quoting and trading
 * `swapMints` pair
 *
 * @param swapMints
 */
export async function infForSwap(
  rpc: Rpc<SolanaRpcApi>,
  swapMints: PkPair
): Promise<Inf> {
  initSyncEmbed();

  const pks = initPks() as Address[];
  const initAccs = await fetchAccountMap(rpc, pks);
  const inf = init(initAccs, SPL_POOL_ACCOUNTS);
  const updateAddrs = accountsToUpdateForTrade(inf, swapMints) as Address[];
  const updateAccs = await fetchAccountMap(rpc, updateAddrs);
  updateForTrade(inf, swapMints, updateAccs);
  return inf;
}

/**
 * Initializes, updates and returns an `Inf` that is ready for rebalancing
 * `swapMints` pair
 *
 * @param rebalanceMints
 */
export async function infForRebalance(
  rpc: Rpc<SolanaRpcApi>,
  rebalanceMints: PkPair
): Promise<Inf> {
  initSyncEmbed();

  const pks = initPks() as Address[];
  const initAccs = await fetchAccountMap(rpc, pks);
  const inf = init(initAccs, SPL_POOL_ACCOUNTS);
  const updateAddrs = accountsToUpdateForRebalance(
    inf,
    rebalanceMints
  ) as Address[];
  const updateAccs = await fetchAccountMap(rpc, updateAddrs);
  updateForRebalance(inf, rebalanceMints, updateAccs);
  return inf;
}

export async function expectInfErr<T>(
  f: () => T | Promise<T>,
  expected: InfErrMsg
) {
  try {
    await f();
  } catch (e) {
    expect((e as Error).message).toBe(expected);
    return;
  }
  throw new Error("Expected failure");
}

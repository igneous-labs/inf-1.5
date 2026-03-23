import {
  accountsToUpdateForRebalance,
  accountsToUpdateForTrade,
  cloneInf,
  deserPoolState,
  getPoolState,
  init,
  initPks,
  initSyncEmbed,
  updateForRebalance,
  updateForTrade,
  type Inf,
  type PkPair,
  type PoolStateV2,
} from "@sanctumso/inf1";
import {
  address,
  type Address,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { fetchAccountMap, localRpc } from "./rpc";
import { SPL_POOL_ACCOUNTS } from "./spl";

export const POOL_STATE_ID = address(
  "AYhux5gJzCoeoc1PoJ1VxwPDe22RwcvpHviLDD1oCGvW",
);
export const LST_STATE_LIST_ID = address(
  "Gb7m4daakbVbrFLR33FKMDVMHAprRZ66CSYt4bpFwUgS",
);

/**
 * Initializes, updates and returns an `Inf` that is ready for quoting and trading
 * `swapMints` pair
 *
 * @param swapMints
 */
export async function infForSwap(
  rpc: Rpc<SolanaRpcApi>,
  swapMints: PkPair,
): Promise<Inf> {
  initSyncEmbed();

  const pks = initPks() as Address[];
  const { value: initAccs } = await fetchAccountMap(rpc, pks);
  const inf = init(initAccs, SPL_POOL_ACCOUNTS);
  const updateAddrs = accountsToUpdateForTrade(inf, swapMints) as Address[];
  const { value: updateAccs } = await fetchAccountMap(rpc, updateAddrs);
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
  rebalanceMints: PkPair,
): Promise<Inf> {
  initSyncEmbed();

  const pks = initPks() as Address[];
  const { value: initAccs } = await fetchAccountMap(rpc, pks);
  const inf = init(initAccs, SPL_POOL_ACCOUNTS);
  const updateAddrs = accountsToUpdateForRebalance(
    inf,
    rebalanceMints,
  ) as Address[];
  const { value: updateAccs } = await fetchAccountMap(rpc, updateAddrs);
  updateForRebalance(inf, rebalanceMints, updateAccs);
  return inf;
}

/**
 *
 * @param f
 * @returns the error expected to be thrown
 */
export async function expectInfErr<T>(
  f: () => T | Promise<T>,
): Promise<unknown> {
  try {
    await f();
  } catch (e) {
    return e;
  }
  throw new Error("Expected failure");
}

export function infDeserPoolState(inf: Inf, data: Uint8Array): PoolStateV2 {
  const deserializer = cloneInf(inf);
  deserPoolState(deserializer, data);
  return getPoolState(deserializer);
}

export async function fetchInitInf(): Promise<{ inf: Inf; rpc: Rpc<SolanaRpcApi> }> {
  initSyncEmbed();
  const rpc = localRpc();
  const pks = initPks() as Address[];
  const { value: initAccs } = await fetchAccountMap(rpc, pks);
  const inf = init(initAccs, SPL_POOL_ACCOUNTS);
  return { inf, rpc };
}

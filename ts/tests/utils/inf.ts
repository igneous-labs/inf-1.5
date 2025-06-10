import {
  accountsToUpdateForTrade,
  init,
  initPks,
  updateForTrade,
  type Inf,
  type PkPair,
} from "@sanctumso/inf1";
import { type Rpc, type SolanaRpcApi } from "@solana/kit";
import { fetchAccountMap } from "./rpc";
import { SPL_POOL_ACCOUNTS } from "./spl";

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
  const { poolState: poolStateAddr, lstStateList: lstStateListAddr } =
    initPks();
  const initAccs = await fetchAccountMap(rpc, [
    poolStateAddr,
    lstStateListAddr,
  ]);
  const inf = init(
    {
      poolState: initAccs.get(poolStateAddr)!,
      lstStateList: initAccs.get(lstStateListAddr)!,
    },
    SPL_POOL_ACCOUNTS
  );
  const updateAccs = await fetchAccountMap(
    rpc,
    accountsToUpdateForTrade(inf, swapMints)
  );
  updateForTrade(inf, swapMints, updateAccs);
  return inf;
}

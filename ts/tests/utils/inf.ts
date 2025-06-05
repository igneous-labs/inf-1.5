import {
  accountsToUpdateForSwap,
  init,
  initPks,
  updateForSwap,
  type InfHandle,
  type PkPair,
} from "@sanctumso/inf1";
import { type Rpc, type SolanaRpcApi } from "@solana/kit";
import { fetchAccountMap } from "./rpc";
import { SPL_POOL_ACCOUNTS } from "./spl";

/**
 * Initializes, updates and returns an `InfHandle` that is ready for quoting and trading
 * `swapMints` pair
 *
 * @param swapMints
 */
export async function infForSwap(
  rpc: Rpc<SolanaRpcApi>,
  swapMints: PkPair
): Promise<InfHandle> {
  const { poolState: poolStateAddr, lstStateList: lstStateListAddr } =
    initPks();
  const initAccs = await fetchAccountMap(rpc, [
    poolStateAddr,
    lstStateListAddr,
  ]);
  const inf = init(
    {
      poolState: initAccs[poolStateAddr],
      lstStateList: initAccs[lstStateListAddr],
    },
    SPL_POOL_ACCOUNTS
  );
  const updateAccs = await fetchAccountMap(
    rpc,
    accountsToUpdateForSwap(inf, swapMints)
  );
  updateForSwap(inf, swapMints, updateAccs);
  return inf;
}

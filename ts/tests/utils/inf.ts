import {
  accountsToUpdateForTrade,
  allInfErrs,
  init,
  initPks,
  initSyncEmbed,
  updateForTrade,
  type Inf,
  type InfErr,
  type PkPair,
} from "@sanctumso/inf1";
import { type Address, type Rpc, type SolanaRpcApi } from "@solana/kit";
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
  initSyncEmbed();

  const pks = initPks() as Address[];
  const initAccs = await fetchAccountMap(rpc, pks);
  const inf = init(initAccs, SPL_POOL_ACCOUNTS);
  const updateAccs = await fetchAccountMap(
    rpc,
    accountsToUpdateForTrade(inf, swapMints) as Address[]
  );
  updateForTrade(inf, swapMints, updateAccs);
  return inf;
}

/**
 *
 * @param e
 * @returns [InfErr, rest of error message]
 */
export function parseInfErr(e: unknown): [InfErr, string] {
  if (!(e instanceof Error)) {
    throw new Error("not Error", { cause: e });
  }

  const i = e.message.indexOf(":");
  if (i < 0) {
    console.log(i);
    throw new Error("Not a InfErr", { cause: e });
  }
  const code = e.message.substring(0, i);
  const rest = e.message.substring(i + 1);
  if (!assertInfErr(code)) {
    throw new Error(`Invalid InfErr code ${code}`, { cause: e });
  }
  return [code, rest];
}

function assertInfErr(code: string): code is InfErr {
  return (allInfErrs() as readonly string[]).includes(code);
}

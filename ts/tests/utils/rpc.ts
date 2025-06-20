import type { AccountMap } from "@sanctumso/inf1";
import {
  assertAccountsExist,
  createSolanaRpc,
  fetchEncodedAccounts,
  type Address,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";

export function localRpc(): Rpc<SolanaRpcApi> {
  return createSolanaRpc("http://localhost:8899");
}

export async function fetchAccountMap(
  rpc: Rpc<SolanaRpcApi>,
  accounts: Address[]
): Promise<AccountMap> {
  const fetched = await fetchEncodedAccounts(rpc, accounts);
  assertAccountsExist(fetched);
  return new Map(
    fetched.map(({ address, data, programAddress }) => [
      address,
      { data, owner: programAddress },
    ])
  );
}

import type { AccountMap } from "@sanctumso/inf1";
import {
  createSolanaRpc,
  getBase64Encoder,
  type Address,
  type Rpc,
  type SolanaRpcApi,
  type SolanaRpcResponse,
} from "@solana/kit";

export function localRpc(): Rpc<SolanaRpcApi> {
  return createSolanaRpc("http://localhost:8899");
}

/**
 *
 * @param rpc
 * @param accounts
 * @returns
 * @throws if any of the account in `accounts` dont exist
 */
export async function fetchAccountMap(
  rpc: Rpc<SolanaRpcApi>,
  accounts: Address[],
): Promise<SolanaRpcResponse<AccountMap>> {
  const { context, value } = await rpc.getMultipleAccounts(accounts).send();
  const zipped = value.map((v, i) => [accounts[i], v] as const);
  return {
    context,
    value: new Map(
      zipped.map(([address, v]) => {
        if (v == null) {
          throw new Error(`missing account ${address}`);
        }
        return [
          address,
          {
            data: new Uint8Array(getBase64Encoder().encode(v.data[0])),
            owner: v.owner,
          },
        ];
      }),
    ),
  };
}

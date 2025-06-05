import type { AccountMap } from "@sanctumso/inf1";
import {
  createSolanaRpc,
  getBase64Encoder,
  type Address,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";

export function localRpc(): Rpc<SolanaRpcApi> {
  return createSolanaRpc("http://localhost:8899");
}

/**
 *
 * @param rpc
 * @param accounts
 *
 * @returns
 *
 * @throws if any account in `accounts` does not exist
 */
export async function fetchAccountMap(
  rpc: Rpc<SolanaRpcApi>,
  accounts: string[]
): Promise<AccountMap> {
  return Object.fromEntries(
    await Promise.all(
      accounts.map(async (account) => {
        const { value } = await rpc
          .getAccountInfo(account as Address, {
            encoding: "base64",
          })
          .send();
        const v = value!;
        return [
          account,
          {
            data: new Uint8Array(getBase64Encoder().encode(v.data[0])),
            owner: v.owner,
          },
        ];
      })
    )
  );
}

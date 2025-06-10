import type { Account, AccountMap } from "@sanctumso/inf1";
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

export async function fetchAccountMap(
  rpc: Rpc<SolanaRpcApi>,
  accounts: string[]
): Promise<AccountMap> {
  const map = new Map<string, Account>();
  await Promise.all(
    accounts.map(async (account) => {
      const accountInfo = await rpc
        .getAccountInfo(account as Address, {
          encoding: "base64",
        })
        .send();
      const acc = accountInfo.value!;
      map.set(account, {
        data: new Uint8Array(getBase64Encoder().encode(acc.data[0])),
        owner: acc.owner,
      });
    })
  );
  return map;
}

import {
  cloneInf,
  deserPoolState,
  getPoolState,
  init,
  initPks,
  withdrawProtocolFeesV2IxRaw,
  type PoolStateV2,
} from "@sanctumso/inf1";
import { INF_MINT, mintSupply, testFixturesTokenAcc, tokenAccBalance } from "./token";
import { fetchAccountMap, ixsToSimTx, localRpc, mapTup, POOL_STATE_ID, SPL_POOL_ACCOUNTS } from ".";
import { address, getBase64Encoder, type Address } from "@solana/kit";
import { expect } from "vitest";

export async function withdrawProtocolFeesV2BasicTest(withdrawToAccountFixture: string): Promise<{
  poolStateBefore: PoolStateV2;
  poolStateAfter: PoolStateV2;
  infMinted: bigint;
  infWithdrawn: bigint;
}> {
  // taken from accounts.test
  const PROTOCOL_FEE_BENEFICIARY = "EeQmNqm1RcQnee8LTyx6ccVG9FnR8TezQuw2JXq2LC1T";
  // token program
  const TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

  const rpc = localRpc();
  const pks = initPks() as Address[];
  const { value: initAccs } = await fetchAccountMap(rpc, pks);
  const inf = init(initAccs, SPL_POOL_ACCOUNTS);
  const poolStateBefore = getPoolState(inf);

  const { addr: withdrawTo } = testFixturesTokenAcc(withdrawToAccountFixture);
  const addresses: Address[] = [withdrawTo, POOL_STATE_ID, INF_MINT];
  const { value: before } = await fetchAccountMap(rpc, addresses);

  const infMintSupplyBefore = mintSupply(before.get(INF_MINT)!.data);
  const withdrawToBalanceBefore = tokenAccBalance(before.get(withdrawTo)!.data);

  const ix = withdrawProtocolFeesV2IxRaw({
    protocolFeeBeneficiary: PROTOCOL_FEE_BENEFICIARY,
    withdrawTo,
    infMint: INF_MINT,
    tokenProgram: TOKEN_PROGRAM_ID,
  });

  const tx = ixsToSimTx(address(PROTOCOL_FEE_BENEFICIARY), [ix]);

  const {
    value: { err, accounts: after, logs },
  } = await rpc
    .simulateTransaction(tx, {
      accounts: {
        addresses,
        encoding: "base64",
      },
      encoding: "base64",
      sigVerify: false,
      replaceRecentBlockhash: true,
    })
    .send();

  const debugMsg = `tx: ${tx}\nlogs:\n${(logs ?? []).join("\n")}\n`;
  expect(err, debugMsg).toBeNull();

  const [withdrawToDataAfter, poolStateDataAfter, infMintDataAfter] = mapTup(
    [...Array(addresses.length).keys()],
    (i) => new Uint8Array(getBase64Encoder().encode(after[i]!.data[0])),
  );

  // clone Inf for deser to not affect with original Inf state
  const deserializer = cloneInf(inf);
  deserPoolState(deserializer, poolStateDataAfter);
  const poolStateAfter = getPoolState(deserializer);

  const infMintSupplyAfter = mintSupply(infMintDataAfter);
  const infMinted = infMintSupplyAfter - infMintSupplyBefore;

  const withdrawToBalanceAfter = tokenAccBalance(withdrawToDataAfter);
  const infWithdrawn = withdrawToBalanceAfter - withdrawToBalanceBefore;

  return {
    poolStateBefore,
    poolStateAfter,
    infMinted,
    infWithdrawn,
  };
}

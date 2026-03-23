import {
  getPoolState,
  withdrawProtocolFeesV2IxRaw,
} from "@sanctumso/inf1";
import { INF_MINT, mintSupply, testFixturesTokenAcc, tokenAccBalance } from "./token";
import { fetchAccountMap, fetchInitInf, infDeserPoolState, ixsToSimTx, mapTup, POOL_STATE_ID } from ".";
import { address, getBase64Encoder, type Address } from "@solana/kit";
import { expect } from "vitest";

export async function withdrawProtocolFeesV2BasicTest(withdrawToAccountFixture: string) {
  // token program
  const TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

  const { inf, rpc } = await fetchInitInf();
  const poolStateBefore = getPoolState(inf);

  const protocolFeeBeneficiary = poolStateBefore.protocolFeeBeneficiary;

  const { addr: withdrawTo } = testFixturesTokenAcc(withdrawToAccountFixture);
  const addresses: Address[] = [withdrawTo, POOL_STATE_ID, INF_MINT];
  const { value: before } = await fetchAccountMap(rpc, addresses);

  const infMintSupplyBefore = mintSupply(before.get(INF_MINT)!.data);
  const withdrawToBalanceBefore = tokenAccBalance(before.get(withdrawTo)!.data);

  const ix = withdrawProtocolFeesV2IxRaw({
    protocolFeeBeneficiary,
    withdrawTo,
    infMint: INF_MINT,
    tokenProgram: TOKEN_PROGRAM_ID,
  });

  const tx = ixsToSimTx(address(protocolFeeBeneficiary), [ix]);

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
  const poolStateAfter = infDeserPoolState(inf, poolStateDataAfter);

  const infMintSupplyAfter = mintSupply(infMintDataAfter);
  const infMinted = infMintSupplyAfter - infMintSupplyBefore;

  const withdrawToBalanceAfter = tokenAccBalance(withdrawToDataAfter);
  const infWithdrawn = withdrawToBalanceAfter - withdrawToBalanceBefore;

  expect(poolStateBefore.protocolFeeLamports).toBeGreaterThan(0n);
  expect(poolStateAfter.protocolFeeLamports).toEqual(0n);
  expect(infWithdrawn).toBeGreaterThan(0n);
  expect(infWithdrawn).toEqual(infMinted);
}

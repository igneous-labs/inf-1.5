import {
  findPoolReservesAta,
  quoteRebalance,
  rebalanceIxs,
  type RebalanceArgs,
  type RebalanceIxs,
  type RebalanceQuote,
} from "@sanctumso/inf1";
import { infForRebalance } from "./inf";
import { mapTup } from "./ops";
import { fetchAccountMap, localRpc } from "./rpc";
import { testFixturesTokenAcc, tokenAccBalance } from "./token";
import {
  address,
  getBase64Encoder,
  type Address,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { getTransferInstruction } from "@solana-program/token";
import { ixsToSimTx } from "./tx";
import { expect } from "vitest";

export async function rebalanceBasicTest(
  out: bigint,
  tokenAccFixtures: { inp: string; out: string }
) {
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpDonorToken, owner: inpDonor, mint: inpMint },
    { addr: outTokenAcc, mint: outMint },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const mints = { inp: inpMint, out: outMint };

  const rpc = localRpc();
  const inf = await infForRebalance(rpc, mints);

  const quote = quoteRebalance(inf, {
    out,
    mints,
  });
  const rebalanceArgs: RebalanceArgs = {
    out,
    // Make these limits a no-op to allow
    // program to pass but assert to fail if quote does not match
    // actual rebalance result
    minStartingOutLst: 0n,
    maxStartingInpLst: 18_446_744_073_709_551_615n,
    mints,
    withdrawTo: outTokenAcc,
  };
  const ixs = rebalanceIxs(inf, rebalanceArgs);
  await simDonateAssertQuoteMatchesRebalance(rpc, quote, rebalanceArgs, ixs, {
    inpDonorToken,
    inpDonor,
  });
}

export async function simDonateAssertQuoteMatchesRebalance(
  rpc: Rpc<SolanaRpcApi>,
  { inp, out, mints: { inp: inpMint, out: outMint } }: RebalanceQuote,
  { withdrawTo }: RebalanceArgs,
  { start, end }: RebalanceIxs,
  { inpDonorToken, inpDonor }: { inpDonorToken: Address; inpDonor: Address }
) {
  // `addresses` layout:
  // - inpDonorToken
  // - withdrawTo
  // - inp pool reserves
  // - out pool reserves
  const [inpPoolAcc, outPoolAcc] = mapTup([inpMint, outMint], (mint) =>
    address(findPoolReservesAta(mint)[0])
  );
  const addresses = [
    inpDonorToken,
    withdrawTo,
    inpPoolAcc,
    outPoolAcc,
  ] as Address[];

  const befRebalance = await fetchAccountMap(rpc, addresses);

  const [
    inpDonorTokenBalanceBef,
    withdrawToBalanceBef,
    inpPoolAccBalanceBef,
    outPoolAccBalanceBef,
  ] = mapTup(addresses, (addr) =>
    tokenAccBalance(befRebalance.get(addr)!.data)
  );

  const tx = ixsToSimTx(address(inpDonor), [
    start,
    getTransferInstruction({
      source: inpDonorToken,
      destination: inpPoolAcc,
      amount: inp,
      authority: inpDonor,
    }),
    end,
  ]);

  const {
    value: { err, accounts: aftSwap, logs },
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
  const debugMsg = `tx: ${tx}\nlogs:\n` + (logs ?? []).join("\n") + "\n";

  expect(err, debugMsg).toBeNull();

  const [
    inpDonorTokenBalanceAft,
    withdrawToBalanceAft,
    inpPoolAccBalanceAft,
    outPoolAccBalanceAft,
  ] = mapTup(addresses, (_addr, i) =>
    tokenAccBalance(
      new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
    )
  );

  expect(inpPoolAccBalanceAft - inpPoolAccBalanceBef).toEqual(inp);
  expect(inpDonorTokenBalanceBef - inpDonorTokenBalanceAft).toEqual(inp);
  expect(outPoolAccBalanceBef - outPoolAccBalanceAft).toEqual(out);
  expect(withdrawToBalanceAft - withdrawToBalanceBef).toEqual(out);
}

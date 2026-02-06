import {
  findPoolReservesAta,
  Inf,
  quoteTradeExactIn,
  quoteTradeExactOut,
  tradeExactInIx,
  tradeExactOutIx,
  cloneInf,
  type Instruction,
  type Quote,
  type TradeArgs,
  deserPoolState,
  type PoolStateV2,
  getPoolState,
} from "@sanctumso/inf1";
import {
  address,
  getBase64Encoder,
  type Address,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { expect } from "vitest";
import {
  INF_MINT,
  mintSupply,
  testFixturesTokenAcc,
  tokenAccBalance,
} from "./token";
import { fetchAccountMap, localRpc } from "./rpc";
import { infForSwap, ixsToSimTx, mapTup, POOL_STATE_ID } from ".";

export async function tradeExactInBasicTest(
  amt: bigint,
  tokenAccFixtures: { inp: string; out: string },
) {
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpTokenAcc, owner: inpTokenAccOwner, mint: inpMint },
    { addr: outTokenAcc, mint: outMint },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const mints = { inp: inpMint, out: outMint };

  const rpc = localRpc();
  const inf = await infForSwap(rpc, mints);

  await simAssertQuoteMatchesTrade(
    rpc,
    inf,
    {
      // use 0n instead of quote.out
      // to allow program to pass but assert to fail
      // if quote does not match actual swap result
      limit: 0n,
      amt,
      mints,
      signer: inpTokenAccOwner,
      tokenAccs: {
        inp: inpTokenAcc,
        out: outTokenAcc,
      },
    },
    "ExactIn",
  );
}

export async function tradeExactOutBasicTest(
  amt: bigint,
  tokenAccFixtures: { inp: string; out: string },
) {
  const { inp: inpTokenAccName, out: outTokenAccName } = tokenAccFixtures;
  const [
    { addr: inpTokenAcc, owner: inpTokenAccOwner, mint: inpMint },
    { addr: outTokenAcc, mint: outMint },
  ] = mapTup([inpTokenAccName, outTokenAccName], testFixturesTokenAcc);
  const mints = { inp: inpMint, out: outMint };

  const rpc = localRpc();
  const inf = await infForSwap(rpc, mints);

  await simAssertQuoteMatchesTrade(
    rpc,
    inf,
    {
      // use u64::MAX instead of quote.inp
      // to allow program to pass but assert to fail
      // if quote does not match actual swap result
      limit: 18_446_744_073_709_551_615n,
      amt,
      mints,
      signer: inpTokenAccOwner,
      tokenAccs: {
        inp: inpTokenAcc,
        out: outTokenAcc,
      },
    },
    "ExactOut",
  );
}

function infDeserPoolState(inf: Inf, data: Uint8Array): PoolStateV2 {
  const deserializer = cloneInf(inf);
  deserPoolState(deserializer, data);
  return getPoolState(deserializer);
}

export async function simAssertQuoteMatchesTrade(
  rpc: Rpc<SolanaRpcApi>,
  inf: Inf,
  args: TradeArgs,
  dir: "ExactIn" | "ExactOut",
) {
  const {
    amt,
    signer,
    tokenAccs: { inp: inpTokenAcc, out: outTokenAcc },
    mints,
  } = args;
  const { inp: inpMint, out: outMint } = mints;

  let ix: Instruction;
  switch (dir) {
    case "ExactIn":
      ix = tradeExactInIx(inf, args);
      break;
    case "ExactOut":
      ix = tradeExactOutIx(inf, args);
      break;
  }

  // // for debugging AccountMissing err
  // for (const { address } of ix.accounts) {
  //   const { value } = await rpc
  //     .getAccountInfo(address as Address, {
  //       encoding: "base64",
  //     })
  //     .send();
  //   if (value == null) {
  //     throw new Error(`AccountMissing ${address}`);
  //   }
  // }

  // `addresses` layout:
  // - inpTokenAcc
  // - outTokenAcc
  // - inp pool acc: either INF mint if removeLiqudiity or input pool reserves otherwise
  // - out pool acc: either INF mint if addLiquidity or output pool reserves otherwise
  // - poolState
  const addresses = [inpTokenAcc, outTokenAcc] as Address[];

  const [inpPoolAcc, outPoolAcc] = mapTup([inpMint, outMint], (mint) => {
    if (mint === INF_MINT) {
      return INF_MINT;
    } else {
      return address(findPoolReservesAta(mint)[0]);
    }
  });
  addresses.push(inpPoolAcc, outPoolAcc, POOL_STATE_ID);

  const { value: befSwap } = await fetchAccountMap(rpc, addresses);

  const [inpTokenAccBalanceBef, outTokenAccBalanceBef] = mapTup(
    [inpTokenAcc, outTokenAcc],
    (addr) => tokenAccBalance(befSwap.get(addr)!.data),
  );
  const poolStateBef = infDeserPoolState(inf, befSwap.get(POOL_STATE_ID)!.data);
  const [inpPoolAmtBef, outPoolAmtBef] = mapTup(
    [
      [inpMint, inpPoolAcc],
      [outMint, outPoolAcc],
    ],
    ([mint, acc]) => {
      const data = befSwap.get(acc)!.data;
      if (mint === INF_MINT) {
        // use negative mint supply so that the diffs are in the same direction
        // inp=LP -> supply goes down but regular swaps have pool balance increasing
        return -mintSupply(data);
      } else {
        return tokenAccBalance(data);
      }
    },
  );

  const tx = ixsToSimTx(address(signer), [ix]);
  const {
    context: { slot: aftSlot },
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
    inpTokenAccDataAft,
    outTokenAccDataAft,
    inpPoolAccDataAft,
    outPoolAccDataAft,
  ] = mapTup(
    [...Array(addresses.length).keys()],
    (i) => new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0])),
  );
  const [inpTokenAccBalanceAft, outTokenAccBalanceAft] = mapTup(
    [inpTokenAccDataAft, outTokenAccDataAft],
    (d) => tokenAccBalance(d),
  );
  const [inpPoolAmtAft, outPoolAmtAft] = mapTup(
    [
      [inpMint, inpPoolAccDataAft],
      [outMint, outPoolAccDataAft],
    ] as const,
    ([mint, data]) => {
      if (mint === INF_MINT) {
        return -mintSupply(data);
      } else {
        return tokenAccBalance(data);
      }
    },
  );

  const slotLookahead = aftSlot - poolStateBef.lastReleaseSlot;
  if (slotLookahead < 0) {
    throw new Error(
      `lookahead ${slotLookahead} < 0. bef:${poolStateBef.lastReleaseSlot}. aft:${aftSlot})`,
    );
  }

  let quote: Quote;
  switch (dir) {
    case "ExactIn":
      quote = quoteTradeExactIn(inf, {
        amt,
        mints,
        slotLookahead,
      });
      break;
    case "ExactOut":
      quote = quoteTradeExactOut(inf, {
        amt,
        mints,
        slotLookahead,
      });
      break;
  }

  expect(inpTokenAccBalanceBef - inpTokenAccBalanceAft).toEqual(quote.inp);
  expect(outTokenAccBalanceAft - outTokenAccBalanceBef).toEqual(quote.out);
  expect(inpPoolAmtAft - inpPoolAmtBef).toEqual(quote.inp);
  expect(outPoolAmtBef - outPoolAmtAft).toEqual(quote.out);
}

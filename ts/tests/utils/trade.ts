import {
  findPoolReservesAta,
  findProtocolFeeAccumulatorAta,
  quoteTradeExactIn,
  quoteTradeExactOut,
  tradeExactInIx,
  tradeExactOutIx,
  type B58PK,
  type Instruction,
  type Quote,
  type TradeArgs,
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

  const quote = quoteTradeExactIn(inf, {
    amt,
    mints,
    slotLookahead: 0n,
  });
  const tradeArgs = {
    amt,
    // use 0n instead of quote.out
    // to allow program to pass but assert to fail
    // if quote does not match actual swap result
    limit: 0n,
    mints,
    signer: inpTokenAccOwner,
    tokenAccs: {
      inp: inpTokenAcc,
      out: outTokenAcc,
    },
  };
  const ix = tradeExactInIx(inf, tradeArgs);

  await simAssertQuoteMatchesTrade(rpc, quote, tradeArgs, ix);
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

  const quote = quoteTradeExactOut(inf, {
    amt,
    mints,
    slotLookahead: 0n,
  });
  const tradeArgs = {
    amt,
    // use u64::MAX instead of quote.inp
    // to allow program to pass but assert to fail
    // if quote does not match actual swap result
    limit: 18_446_744_073_709_551_615n,
    mints,
    signer: inpTokenAccOwner,
    tokenAccs: {
      inp: inpTokenAcc,
      out: outTokenAcc,
    },
  };
  const ix = tradeExactOutIx(inf, tradeArgs);

  await simAssertQuoteMatchesTrade(rpc, quote, tradeArgs, ix);
}

/**
 *
 * @param rpc
 * @param quote
 * @param tradeArgs
 * @param ix
 */
export async function simAssertQuoteMatchesTrade(
  rpc: Rpc<SolanaRpcApi>,
  {
    inp: inpAmt,
    out: outAmt,
    fee,
    mints: { inp: inpMint, out: outMint },
  }: Quote,
  { signer, tokenAccs: { inp: inpTokenAcc, out: outTokenAcc } }: TradeArgs,
  ix: Instruction,
) {
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
  // - poolState
  // - inp pool acc: either INF mint if removeLiqudiity or input pool reserves otherwise
  // - out pool acc: either INF mint if addLiquidity or output pool reserves otherwise
  const addresses = [inpTokenAcc, outTokenAcc, POOL_STATE_ID] as Address[];

  const [inpPoolAcc, outPoolAcc] = mapTup([inpMint, outMint], (mint) => {
    if (mint === INF_MINT) {
      return INF_MINT;
    } else {
      return address(findPoolReservesAta(mint)[0]);
    }
  });
  addresses.push(inpPoolAcc, outPoolAcc);

  const befSwap = await fetchAccountMap(rpc, addresses);

  const [inpTokenAccBalanceBef, outTokenAccBalanceBef] = mapTup(
    [inpTokenAcc, outTokenAcc],
    (addr) => tokenAccBalance(befSwap.get(addr)!.data),
  );
  const poolStateAccDataBef = befSwap.get(POOL_STATE_ID)!.data;
  const [inpPoolAmtBef, outPoolAmtBef] = mapTup(
    [
      [inpMint, inpPoolAcc],
      [outMint, outPoolAcc],
    ],
    ([mint, acc]) => {
      const data = befSwap.get(acc)!.data;
      if (mint === INF_MINT) {
        return mintSupply(data);
      } else {
        return tokenAccBalance(data);
      }
    },
  );

  const tx = ixsToSimTx(address(signer), [ix]);
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
    inpTokenAccDataAft,
    outTokenAccDataAft,
    poolStateAccDataAft,
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
        return mintSupply(data);
      } else {
        return tokenAccBalance(data);
      }
    },
  );

  expect(inpTokenAccBalanceBef - inpTokenAccBalanceAft).toEqual(inpAmt);
  expect(outTokenAccBalanceAft - outTokenAccBalanceBef).toEqual(outAmt);
  expect(inpPoolAmtBef - inpPoolAmtAft).toEqual(inpAmt);
  expect(outPoolAmtAft - outPoolAmtBef).toEqual(outAmt);
}

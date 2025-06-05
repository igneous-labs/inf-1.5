import {
  findPoolReservesAta,
  findProtocolFeeAccumulatorAta,
  type B58PK,
  type Instruction,
  type Quote,
  type TradeArgs,
} from "@sanctumso/inf1";
import {
  address,
  appendTransactionMessageInstruction,
  blockhash,
  compileTransaction,
  createTransactionMessage,
  getBase64EncodedWireTransaction,
  getBase64Encoder,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  type Address,
  type Base64EncodedWireTransaction,
  type IInstruction,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import { expect } from "vitest";
import { INF_MINT, mintSupply, tokenAccBalance } from "./token";
import { fetchAccountMap } from "./rpc";
import { mapTup } from ".";

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
    protocolFee,
    feeMint,
    mints: { inp: inpMint, out: outMint },
  }: Quote,
  { signer, tokenAccs: { inp: inpTokenAcc, out: outTokenAcc } }: TradeArgs,
  ix: Instruction
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
  // - protocolFeeAccumulator
  // - inp pool acc: either INF mint if removeLiqudiity or input pool reserves otherwise
  // - out pool acc: either INF mint if addLiquidity or output pool reserves otherwise
  const addresses = [inpTokenAcc, outTokenAcc] as Address[];

  let pfMint: B58PK;
  switch (feeMint) {
    case "inp":
      pfMint = inpMint;
      break;
    case "out":
      pfMint = outMint;
      break;
  }
  const pfAccumAddr = address(findProtocolFeeAccumulatorAta(pfMint)[0]);
  addresses.push(pfAccumAddr);

  const [inpPoolAcc, outPoolAcc] = mapTup([inpMint, outMint], (mint) => {
    if (mint === INF_MINT) {
      return INF_MINT;
    } else {
      return address(findPoolReservesAta(mint)[0]);
    }
  });
  addresses.push(inpPoolAcc, outPoolAcc);

  const befSwap = await fetchAccountMap(rpc, addresses);

  const [inpTokenAccBalanceBef, outTokenAccBalanceBef, pfAccumBalanceBef] =
    mapTup([inpTokenAcc, outTokenAcc, pfAccumAddr], (addr) =>
      tokenAccBalance(befSwap[addr].data)
    );
  const [inpPoolAmtBef, outPoolAmtBef] = mapTup(
    [
      [inpMint, inpPoolAcc],
      [outMint, outPoolAcc],
    ],
    ([mint, acc]) => {
      const data = befSwap[acc].data;
      if (mint === INF_MINT) {
        return mintSupply(data);
      } else {
        return tokenAccBalance(data);
      }
    }
  );

  const {
    value: { err, accounts: aftSwap, logs },
  } = await rpc
    .simulateTransaction(tradeIxToSimTx(address(signer), ix), {
      accounts: {
        addresses,
        encoding: "base64",
      },
      encoding: "base64",
      sigVerify: false,
      replaceRecentBlockhash: true,
    })
    .send();
  const logsMsg = (logs ?? []).join("\n") + "\n";

  expect(err, logsMsg).toBeNull();

  const [inpTokenAccBalanceAft, outTokenAccBalanceAft, pfAccumBalanceAft] =
    mapTup([0, 1, 2], (i) =>
      tokenAccBalance(
        new Uint8Array(getBase64Encoder().encode(aftSwap[i]!.data[0]))
      )
    );
  const [inpPoolAmtAft, outPoolAmtAft] = mapTup(
    [
      [inpMint, 3],
      [outMint, 4],
    ] as const,
    ([mint, i]) => {
      const data = new Uint8Array(
        getBase64Encoder().encode(aftSwap[i]!.data[0])
      );
      if (mint === INF_MINT) {
        return mintSupply(data);
      } else {
        return tokenAccBalance(data);
      }
    }
  );

  expect(inpTokenAccBalanceBef - inpTokenAccBalanceAft).toEqual(inpAmt);
  expect(outTokenAccBalanceAft - outTokenAccBalanceBef).toEqual(outAmt);
  expect(pfAccumBalanceAft - pfAccumBalanceBef).toEqual(protocolFee);
  if (inpPoolAcc === INF_MINT) {
    // RemoveLiquidity: assert token supply decrease
    expect(inpPoolAmtBef - inpPoolAmtAft).toEqual(inpAmt);
  } else {
    // Swap: assert inp reserves balance increase
    expect(inpPoolAmtAft - inpPoolAmtBef).toEqual(inpAmt);
  }
  if (outPoolAcc === INF_MINT) {
    // AddLiquidity: assert token supply increase
    expect(outPoolAmtAft - outPoolAmtBef).toEqual(outAmt);
  } else {
    // Swap: assert out reserves balance decrease
    expect(outPoolAmtBef - outPoolAmtAft).toEqual(outAmt);
  }
}

export function tradeIxToSimTx(
  payer: Address,
  ix: Instruction
): Base64EncodedWireTransaction {
  return pipe(
    createTransactionMessage({ version: 0 }),
    (txm) =>
      appendTransactionMessageInstruction(ix as unknown as IInstruction, txm),
    (txm) => setTransactionMessageFeePayer(payer, txm),
    (txm) =>
      setTransactionMessageLifetimeUsingBlockhash(
        {
          blockhash: blockhash("11111111111111111111111111111111"),
          lastValidBlockHeight: 0n,
        },
        txm
      ),
    compileTransaction,
    getBase64EncodedWireTransaction
  );
}

import type { Instruction } from "@sanctumso/inf1";
import {
  appendTransactionMessageInstructions,
  blockhash,
  compileTransaction,
  createTransactionMessage,
  getBase64EncodedWireTransaction,
  pipe,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  type Address,
  type Base64EncodedWireTransaction,
  type IInstruction,
} from "@solana/kit";

/**
 * Creates a simulatable transaction from the list of instructions
 *
 * - blockhash = null blockhash
 *
 * @param payer
 * @param ix
 * @returns
 */
export function ixsToSimTx(
  payer: Address,
  ixs: Array<Instruction | IInstruction>
): Base64EncodedWireTransaction {
  return pipe(
    createTransactionMessage({ version: 0 }),
    (txm) =>
      appendTransactionMessageInstructions(
        ixs as unknown[] as IInstruction[],
        txm
      ),
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

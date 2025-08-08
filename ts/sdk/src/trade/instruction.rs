use bs58_fixed_wasm::Bs58Array;
use inf1_std::{
    inf1_ctl_core::{
        self,
        instructions::{
            liquidity::{add::AddLiquidityIxData, remove::RemoveLiquidityIxData},
            swap::{exact_in::SwapExactInIxData, exact_out::SwapExactOutIxData},
        },
    },
    instructions::swap::{
        exact_in::{
            swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer, swap_exact_in_ix_keys_owned,
        },
        exact_out::{
            swap_exact_out_ix_is_signer, swap_exact_out_ix_is_writer, swap_exact_out_ix_keys_owned,
        },
    },
    trade::instruction::TradeIxArgs,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

#[allow(deprecated)]
use inf1_std::instructions::liquidity::{
    add::{add_liquidity_ix_is_signer, add_liquidity_ix_is_writer, add_liquidity_ix_keys_owned},
    remove::{
        remove_liquidity_ix_is_signer, remove_liquidity_ix_is_writer,
        remove_liquidity_ix_keys_owned,
    },
};

use crate::{
    err::InfError,
    instruction::{keys_signer_writable_to_metas, Instruction},
    interface::{PkPair, B58PK},
    trade::Pair,
    Inf,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct TradeArgs {
    pub amt: u64,
    pub limit: u64,
    pub mints: PkPair,
    pub signer: B58PK,
    pub token_accs: PkPair,
}

/// @throws
#[wasm_bindgen(js_name = tradeExactInIx)]
pub fn trade_exact_in_ix(
    inf: &mut Inf,
    TradeArgs {
        amt,
        limit,
        mints:
            PkPair {
                inp: Bs58Array(inp_mint),
                out: Bs58Array(out_mint),
            },
        signer: Bs58Array(signer),
        token_accs:
            PkPair {
                inp: Bs58Array(inp_token_acc),
                out: Bs58Array(out_token_acc),
            },
    }: &TradeArgs,
) -> Result<Instruction, InfError> {
    let trade_ix_args = TradeIxArgs {
        amt: *amt,
        limit: *limit,
        mints: &Pair {
            inp: inp_mint,
            out: out_mint,
        },
        signer,
        token_accs: &Pair {
            inp: inp_token_acc,
            out: out_token_acc,
        },
    };

    let lp_token_mint = inf.0.pool().lp_token_mint;
    let ix = if *out_mint == lp_token_mint {
        // add liquidity
        let ix = inf.0.add_liq_ix_mut(&trade_ix_args)?;
        Instruction {
            #[allow(deprecated)]
            accounts: keys_signer_writable_to_metas(
                add_liquidity_ix_keys_owned(&ix.accs).seq(),
                add_liquidity_ix_is_signer(&ix.accs).seq(),
                add_liquidity_ix_is_writer(&ix.accs).seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
            data: ByteBuf::from(AddLiquidityIxData::new(ix.to_full()).as_buf()),
        }
    } else if *inp_mint == lp_token_mint {
        // remove liquidity
        let ix = inf.0.remove_liq_ix_mut(&trade_ix_args)?;
        Instruction {
            #[allow(deprecated)]
            accounts: keys_signer_writable_to_metas(
                remove_liquidity_ix_keys_owned(&ix.accs).seq(),
                remove_liquidity_ix_is_signer(&ix.accs).seq(),
                remove_liquidity_ix_is_writer(&ix.accs).seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
            data: ByteBuf::from(RemoveLiquidityIxData::new(ix.to_full()).as_buf()),
        }
    } else {
        // swap exact in
        let ix = inf.0.swap_exact_in_ix_mut(&trade_ix_args)?;
        Instruction {
            accounts: keys_signer_writable_to_metas(
                swap_exact_in_ix_keys_owned(&ix.accs).seq(),
                swap_exact_in_ix_is_signer(&ix.accs).seq(),
                swap_exact_in_ix_is_writer(&ix.accs).seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
            data: ByteBuf::from(SwapExactInIxData::new(ix.to_full()).as_buf()),
        }
    };
    Ok(ix)
}

/// @throws
#[wasm_bindgen(js_name = tradeExactOutIx)]
pub fn trade_exact_out_ix(
    inf: &mut Inf,
    TradeArgs {
        amt,
        limit,
        mints:
            PkPair {
                inp: Bs58Array(inp_mint),
                out: Bs58Array(out_mint),
            },
        signer: Bs58Array(signer),
        token_accs:
            PkPair {
                inp: Bs58Array(inp_token_acc),
                out: Bs58Array(out_token_acc),
            },
    }: &TradeArgs,
) -> Result<Instruction, InfError> {
    // only SwapExactOut is supported for exact out
    // a lot of repeated code with SwapExactIn here,
    // but keeping them for now to allow for decoupled evolution
    let trade_ix_args = TradeIxArgs {
        amt: *amt,
        limit: *limit,
        mints: &Pair {
            inp: inp_mint,
            out: out_mint,
        },
        signer,
        token_accs: &Pair {
            inp: inp_token_acc,
            out: out_token_acc,
        },
    };
    let ix = inf.0.swap_exact_out_ix_mut(&trade_ix_args)?;
    Ok(Instruction {
        accounts: keys_signer_writable_to_metas(
            swap_exact_out_ix_keys_owned(&ix.accs).seq(),
            swap_exact_out_ix_is_signer(&ix.accs).seq(),
            swap_exact_out_ix_is_writer(&ix.accs).seq(),
        ),
        program_address: B58PK::new(inf1_ctl_core::ID),
        data: ByteBuf::from(SwapExactOutIxData::new(ix.to_full()).as_buf()),
    })
}

use bs58_fixed_wasm::Bs58Array;
use inf1_core::{
    inf1_ctl_core::{
        self,
        accounts::pool_state::PoolState,
        instructions::{
            liquidity::{
                add::{AddLiquidityIxData, NewAddLiquidityIxPreAccsBuilder},
                remove::{NewRemoveLiquidityIxPreAccsBuilder, RemoveLiquidityIxData},
            },
            swap::{
                exact_in::{NewSwapExactInIxPreAccsBuilder, SwapExactInIxData},
                exact_out::{NewSwapExactOutIxPreAccsBuilder, SwapExactOutIxData},
            },
        },
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        typedefs::lst_state::LstState,
    },
    instructions::{
        liquidity::{
            add::{
                add_liquidity_ix_is_signer, add_liquidity_ix_is_writer,
                add_liquidity_ix_keys_owned, AddLiquidityIxAccs, AddLiquidityIxArgs,
            },
            remove::{
                remove_liquidity_ix_is_signer, remove_liquidity_ix_is_writer,
                remove_liquidity_ix_keys_owned, RemoveLiquidityIxAccs, RemoveLiquidityIxArgs,
            },
        },
        swap::{
            exact_in::{
                swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer,
                swap_exact_in_ix_keys_owned, SwapExactInIxAccs, SwapExactInIxArgs,
            },
            exact_out::{
                swap_exact_out_ix_is_signer, swap_exact_out_ix_is_writer,
                swap_exact_out_ix_keys_owned, SwapExactOutIxAccs, SwapExactOutIxArgs,
            },
        },
    },
};
use inf1_svc_ag::inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::TOKEN_PROGRAM;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::missing_svc_data,
    instruction::{keys_signer_writable_to_metas, Instruction},
    interface::B58PK,
    pda::controller::{create_raw_pool_reserves_ata, create_raw_protocol_fee_accumulator_ata},
    trade::{Pair, PkPair},
    utils::try_find_lst_state,
    InfHandle,
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

#[wasm_bindgen(js_name = tradeExactInIx)]
pub fn trade_exact_in_ix(
    inf: &InfHandle,
    TradeArgs {
        amt,
        limit,
        mints:
            PkPair(Pair {
                inp: Bs58Array(inp_mint),
                out: Bs58Array(out_mint),
            }),
        signer: Bs58Array(signer),
        token_accs:
            PkPair(Pair {
                inp: Bs58Array(inp_token_acc),
                out: Bs58Array(out_token_acc),
            }),
    }: &TradeArgs,
) -> Result<Instruction, JsError> {
    let InfHandle {
        pool:
            PoolState {
                lp_token_mint,
                pricing_program,
                ..
            },
        pricing,
        lsts,
        ..
    } = inf;
    let lst_state_list = inf.lst_state_list();

    let ix = if out_mint == lp_token_mint {
        // add liquidity
        let (
            i,
            LstState {
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value_calculator,
                ..
            },
        ) = try_find_lst_state(lst_state_list, inp_mint)?;
        let inp_calc = lsts
            .get(inp_mint)
            .map(|(c, _)| c.as_sol_val_calc_accs())
            .ok_or_else(|| missing_svc_data(inp_mint))?;
        let pricing = pricing.to_price_lp_tokens_to_mint_accs();
        let reserves_addr = create_raw_pool_reserves_ata(inp_mint, pool_reserves_bump);
        let protocol_fee_accumulator_addr =
            create_raw_protocol_fee_accumulator_ata(inp_mint, protocol_fee_accumulator_bump);
        let accs = AddLiquidityIxAccs {
            ix_prefix: NewAddLiquidityIxPreAccsBuilder::start()
                .with_pool_reserves(reserves_addr)
                .with_protocol_fee_accumulator(protocol_fee_accumulator_addr)
                .with_signer(*signer)
                .with_lst_acc(*inp_token_acc)
                .with_lp_acc(*out_token_acc)
                .with_lst_mint(*inp_mint)
                .with_lp_token_mint(*out_mint)
                .with_lst_token_program(TOKEN_PROGRAM)
                .with_lp_token_program(TOKEN_PROGRAM)
                .with_lst_state_list(LST_STATE_LIST_ID)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            lst_calc_prog: sol_value_calculator,
            lst_calc: inp_calc,
            pricing_prog: *pricing_program,
            pricing,
        };
        Instruction {
            accounts: keys_signer_writable_to_metas(
                add_liquidity_ix_keys_owned(&accs).seq(),
                add_liquidity_ix_is_signer(&accs).seq(),
                add_liquidity_ix_is_writer(&accs).seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
            data: (*AddLiquidityIxData::new(
                AddLiquidityIxArgs {
                    // as-safety: i should not > u32::MAX
                    lst_index: i as u32,
                    amount: *amt,
                    min_out: *limit,
                    accs,
                }
                .to_full(),
            )
            .as_buf())
            .into(),
        }
    } else if inp_mint == lp_token_mint {
        // remove liquidity
        let (
            i,
            LstState {
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value_calculator,
                ..
            },
        ) = try_find_lst_state(lst_state_list, out_mint)?;
        let out_calc = lsts
            .get(out_mint)
            .map(|(c, _)| c.as_sol_val_calc_accs())
            .ok_or_else(|| missing_svc_data(out_mint))?;
        let pricing = pricing.to_price_lp_tokens_to_redeem_accs();
        let reserves_addr = create_raw_pool_reserves_ata(out_mint, pool_reserves_bump);
        let protocol_fee_accumulator_addr =
            create_raw_protocol_fee_accumulator_ata(out_mint, protocol_fee_accumulator_bump);
        let accs = RemoveLiquidityIxAccs {
            ix_prefix: NewRemoveLiquidityIxPreAccsBuilder::start()
                .with_pool_reserves(reserves_addr)
                .with_protocol_fee_accumulator(protocol_fee_accumulator_addr)
                .with_signer(*signer)
                .with_lst_acc(*out_token_acc)
                .with_lp_acc(*inp_token_acc)
                .with_lst_mint(*out_mint)
                .with_lp_token_mint(*inp_mint)
                .with_lst_token_program(TOKEN_PROGRAM)
                .with_lp_token_program(TOKEN_PROGRAM)
                .with_lst_state_list(LST_STATE_LIST_ID)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            lst_calc_prog: sol_value_calculator,
            lst_calc: out_calc,
            pricing_prog: *pricing_program,
            pricing,
        };
        Instruction {
            accounts: keys_signer_writable_to_metas(
                remove_liquidity_ix_keys_owned(&accs).seq(),
                remove_liquidity_ix_is_signer(&accs).seq(),
                remove_liquidity_ix_is_writer(&accs).seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
            data: (*RemoveLiquidityIxData::new(
                RemoveLiquidityIxArgs {
                    // as-safety: i should not > u32::MAX
                    lst_index: i as u32,
                    amount: *amt,
                    min_out: *limit,
                    accs,
                }
                .to_full(),
            )
            .as_buf())
            .into(),
        }
    } else {
        // swap
        let [inp_res, out_res]: [Result<_, JsError>; 2] = [inp_mint, out_mint].map(|mint| {
            let (
                i,
                LstState {
                    pool_reserves_bump,
                    protocol_fee_accumulator_bump,
                    sol_value_calculator: calc_addr,
                    ..
                },
            ) = try_find_lst_state(lst_state_list, mint)?;
            let calc = lsts
                .get(mint)
                .map(|(c, _)| c.as_sol_val_calc_accs())
                .ok_or_else(|| missing_svc_data(mint))?;
            let reserves_addr = create_raw_pool_reserves_ata(mint, pool_reserves_bump);
            Ok((
                i,
                calc,
                calc_addr,
                reserves_addr,
                protocol_fee_accumulator_bump,
            ))
        });
        let (inp_i, inp_calc, inp_calc_addr, inp_reserves_addr, _) = inp_res?;
        let (out_i, out_calc, out_calc_addr, out_reserves_addr, out_pf_accum_bump) = out_res?;
        let protocol_fee_accumulator_addr =
            create_raw_protocol_fee_accumulator_ata(out_mint, out_pf_accum_bump);
        let pricing = pricing.to_price_swap_accs(&Pair {
            inp: inp_mint,
            out: out_mint,
        });
        let accs = SwapExactInIxAccs {
            ix_prefix: NewSwapExactInIxPreAccsBuilder::start()
                .with_inp_pool_reserves(inp_reserves_addr)
                .with_out_pool_reserves(out_reserves_addr)
                .with_protocol_fee_accumulator(protocol_fee_accumulator_addr)
                .with_signer(*signer)
                .with_inp_lst_acc(*inp_token_acc)
                .with_out_lst_acc(*out_token_acc)
                .with_inp_lst_mint(*inp_mint)
                .with_out_lst_mint(*out_mint)
                .with_inp_lst_token_program(TOKEN_PROGRAM)
                .with_out_lst_token_program(TOKEN_PROGRAM)
                .with_lst_state_list(LST_STATE_LIST_ID)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            inp_calc_prog: inp_calc_addr,
            inp_calc,
            out_calc_prog: out_calc_addr,
            out_calc,
            pricing_prog: *pricing_program,
            pricing,
        };
        Instruction {
            accounts: keys_signer_writable_to_metas(
                swap_exact_in_ix_keys_owned(&accs).seq(),
                swap_exact_in_ix_is_signer(&accs).seq(),
                swap_exact_in_ix_is_writer(&accs).seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
            data: (*SwapExactInIxData::new(
                SwapExactInIxArgs {
                    // as-safety: i should not > u32::MAX
                    inp_lst_index: inp_i as u32,
                    out_lst_index: out_i as u32,

                    limit: *limit,
                    amount: *amt,
                    accs,
                }
                .to_full(),
            )
            .as_buf())
            .into(),
        }
    };
    Ok(ix)
}

#[wasm_bindgen(js_name = tradeExactOutIx)]
pub fn trade_exact_out_ix(
    inf: &InfHandle,
    TradeArgs {
        amt,
        limit,
        mints:
            PkPair(Pair {
                inp: Bs58Array(inp_mint),
                out: Bs58Array(out_mint),
            }),
        signer: Bs58Array(signer),
        token_accs:
            PkPair(Pair {
                inp: Bs58Array(inp_token_acc),
                out: Bs58Array(out_token_acc),
            }),
    }: &TradeArgs,
) -> Result<Instruction, JsError> {
    // only SwapExactOut is supported for exact out
    let InfHandle {
        pool: PoolState {
            pricing_program, ..
        },
        pricing,
        lsts,
        ..
    } = inf;
    let lst_state_list = inf.lst_state_list();

    // a lot of repeated code with SwapExactIn here,
    // but keeping them for now to allow for decoupled evolution

    let [inp_res, out_res]: [Result<_, JsError>; 2] = [inp_mint, out_mint].map(|mint| {
        let (
            i,
            LstState {
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value_calculator: calc_addr,
                ..
            },
        ) = try_find_lst_state(lst_state_list, mint)?;
        let calc = lsts
            .get(mint)
            .map(|(c, _)| c.as_sol_val_calc_accs())
            .ok_or_else(|| missing_svc_data(mint))?;
        let reserves_addr = create_raw_pool_reserves_ata(mint, pool_reserves_bump);
        Ok((
            i,
            calc,
            calc_addr,
            reserves_addr,
            protocol_fee_accumulator_bump,
        ))
    });
    let (inp_i, inp_calc, inp_calc_addr, inp_reserves_addr, _) = inp_res?;
    let (out_i, out_calc, out_calc_addr, out_reserves_addr, out_pf_accum_bump) = out_res?;
    let protocol_fee_accumulator_addr =
        create_raw_protocol_fee_accumulator_ata(out_mint, out_pf_accum_bump);
    let pricing = pricing.to_price_swap_accs(&Pair {
        inp: inp_mint,
        out: out_mint,
    });
    let accs = SwapExactOutIxAccs {
        ix_prefix: NewSwapExactOutIxPreAccsBuilder::start()
            .with_inp_pool_reserves(inp_reserves_addr)
            .with_out_pool_reserves(out_reserves_addr)
            .with_protocol_fee_accumulator(protocol_fee_accumulator_addr)
            .with_signer(*signer)
            .with_inp_lst_acc(*inp_token_acc)
            .with_out_lst_acc(*out_token_acc)
            .with_inp_lst_mint(*inp_mint)
            .with_out_lst_mint(*out_mint)
            .with_inp_lst_token_program(TOKEN_PROGRAM)
            .with_out_lst_token_program(TOKEN_PROGRAM)
            .with_lst_state_list(LST_STATE_LIST_ID)
            .with_pool_state(POOL_STATE_ID)
            .build(),
        inp_calc_prog: inp_calc_addr,
        inp_calc,
        out_calc_prog: out_calc_addr,
        out_calc,
        pricing_prog: *pricing_program,
        pricing,
    };
    Ok(Instruction {
        accounts: keys_signer_writable_to_metas(
            swap_exact_out_ix_keys_owned(&accs).seq(),
            swap_exact_out_ix_is_signer(&accs).seq(),
            swap_exact_out_ix_is_writer(&accs).seq(),
        ),
        program_address: B58PK::new(inf1_ctl_core::ID),
        data: (*SwapExactOutIxData::new(
            SwapExactOutIxArgs {
                // as-safety: i should not > u32::MAX
                inp_lst_index: inp_i as u32,
                out_lst_index: out_i as u32,

                limit: *limit,
                amount: *amt,
                accs,
            }
            .to_full(),
        )
        .as_buf())
        .into(),
    })
}

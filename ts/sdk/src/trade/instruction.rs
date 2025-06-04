use bs58_fixed_wasm::Bs58Array;
use inf1_core::{
    inf1_ctl_core::{
        self,
        accounts::pool_state::PoolState,
        instructions::liquidity::add::{AddLiquidityIxData, NewAddLiquidityIxPreAccsBuilder},
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        typedefs::lst_state::LstState,
    },
    instructions::liquidity::{
        add::{
            add_liquidity_ix_is_signer, add_liquidity_ix_is_writer, add_liquidity_ix_keys_owned,
            AddLiquidityIxAccs, AddLiquidityIxArgs,
        },
        liquidity_ix_accs_seq,
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
    trade::PkPair,
    utils::{
        create_raw_pool_reserves_ata, create_raw_protocol_fee_accumulator_ata, try_find_lst_state,
    },
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
) -> Result<Instruction, JsError> {
    let InfHandle {
        pool: PoolState { lp_token_mint, .. },
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
                ..
            },
        ) = try_find_lst_state(lst_state_list, out_mint)?;
        let inp_calc = lsts
            .get(inp_mint)
            .map(|(c, _)| c.as_sol_val_calc_accs())
            .ok_or_else(|| missing_svc_data(inp_mint))?;
        let reserves_addr = create_raw_pool_reserves_ata(out_mint, pool_reserves_bump);
        let protocol_fee_accumulator_addr =
            create_raw_protocol_fee_accumulator_ata(out_mint, protocol_fee_accumulator_bump);
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
            lst_calc: inp_calc,
            pricing: pricing.to_price_lp_tokens_to_mint_accs(),
        };
        Instruction {
            accounts: keys_signer_writable_to_metas(
                liquidity_ix_accs_seq(&add_liquidity_ix_keys_owned(&accs)),
                liquidity_ix_accs_seq(&add_liquidity_ix_is_signer(&accs)),
                liquidity_ix_accs_seq(&add_liquidity_ix_is_writer(&accs)),
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
        todo!()
    } else {
        // swap
        todo!()
    };
    Ok(ix)
}

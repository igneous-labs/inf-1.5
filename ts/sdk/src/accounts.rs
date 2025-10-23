use wasm_bindgen::prelude::*;

use crate::{
    err::InfError,
    interface::{LstState, LstStateList, PoolState, B58PK},
    Inf,
};

#[wasm_bindgen(js_name = getPoolState)]
pub fn get_pool_state(inf: &Inf) -> PoolState {
    let inf1_std::inf1_ctl_core::accounts::pool_state::PoolState {
        total_sol_value,
        admin,
        is_disabled,
        is_rebalancing,
        lp_protocol_fee_bps,
        lp_token_mint,
        padding: _,
        pricing_program,
        protocol_fee_beneficiary,
        rebalance_authority,
        trading_protocol_fee_bps,
        version,
    } = inf.0.pool;
    PoolState {
        total_sol_value,
        admin: B58PK::new(admin),
        is_disabled,
        is_rebalancing,
        lp_protocol_fee_bps,
        lp_token_mint: B58PK::new(lp_token_mint),
        pricing_program: B58PK::new(pricing_program),
        protocol_fee_beneficiary: B58PK::new(protocol_fee_beneficiary),
        rebalance_authority: B58PK::new(rebalance_authority),
        trading_protocol_fee_bps,
        version,
    }
}

/// @throws
#[wasm_bindgen(js_name = getLstStateList)]
pub fn get_lst_state_list(inf: &Inf) -> Result<LstStateList, InfError> {
    let states: Vec<LstState> = inf
        .0
        .try_lst_state_list()?
        .iter()
        .map(|packed| {
            let inf1_std::inf1_ctl_core::typedefs::lst_state::LstState {
                is_input_disabled,
                mint,
                padding: _,
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value,
                sol_value_calculator,
            } = packed.into_lst_state();
            LstState {
                is_input_disabled,
                mint: B58PK::new(mint),
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value,
                sol_value_calculator: B58PK::new(sol_value_calculator),
            }
        })
        .collect();

    Ok(LstStateList { states })
}

use bs58_fixed_wasm::Bs58Array;
use inf1_core::{
    inf1_ctl_core::{accounts::pool_state::PoolState, typedefs::lst_state::LstState},
    inf1_svc_core::traits::SolValCalc,
    quote::liquidity::{
        add::{quote_add_liq, AddLiqQuote, AddLiqQuoteArgs},
        remove::{quote_remove_liq, RemoveLiqQuote, RemoveLiqQuoteArgs},
    },
    sync::SyncSolVal,
};
use inf1_pp_flatfee_core::instructions::pricing::lp::redeem::FlatFeeRedeemLpAccs;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{generic_err, missing_svc_data},
    missing_acc_err,
    trade::PkPair,
    utils::try_find_lst_state,
    InfHandle,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum FeeMint {
    Inp,
    Out,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct QuoteArgs {
    pub amt: u64,
    pub mints: PkPair,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// Amount of input tokens given by the user to the pool,
    /// after fees. This is exactly the amount of tokens that
    /// will leave the user's wallet.
    pub inp: u64,

    /// Amount of output tokens returned by the pool to the user,
    /// after fees. This is exactly the amount of tokens that
    /// will enter the user's wallet.
    pub out: u64,

    /// The amount of fee accrued to pool LPs,
    /// accumulated in the pool reserves.
    ///
    /// Which mint it is denoted in (whether inp_mint or out_mint)
    /// depends on value of `self.fee_mint`
    pub lp_fee: u64,

    /// The amount of fee accrued to the protocol,
    /// to be transferred to the protocol fee accumulator account.
    ///
    /// Which mint it is denoted in (whether inp_mint or out_mint)
    /// depends on value of `self.fee_mint`
    pub protocol_fee: u64,

    pub fee_mint: FeeMint,

    pub mints: PkPair,
}

#[wasm_bindgen(js_name = quoteTradeExactIn)]
pub fn quote_trade_exact_in(
    inf: &InfHandle,
    QuoteArgs { amt, mints }: &QuoteArgs,
) -> Result<Quote, JsError> {
    let InfHandle {
        pool:
            PoolState {
                lp_token_mint,
                total_sol_value,
                lp_protocol_fee_bps,
                ..
            },
        lp_token_supply,
        lsts,
        pricing,
        ..
    } = inf;
    let PkPair {
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    } = mints;

    let quote = if out_mint == lp_token_mint {
        // add liquidity
        let lp_token_supply = lp_token_supply.ok_or_else(|| missing_acc_err(lp_token_mint))?;
        let (inp_calc, inp_reserves) = lsts
            .get(inp_mint)
            .and_then(|(c, r)| {
                let calc = c.as_sol_val_calc()?;
                let reserves = r.as_ref()?;
                Some((calc, reserves))
            })
            .ok_or_else(|| missing_svc_data(inp_mint))?;

        // need to perform a manual SyncSolValue of inp mint first
        // in case pool_total_sol_value is stale
        let lst_state_list = inf.lst_state_list();
        let (
            _i,
            LstState {
                sol_value: old_sol_val,
                ..
            },
        ) = try_find_lst_state(lst_state_list, inp_mint)?;
        let new_sol_val = *inp_calc
            .lst_to_sol(inp_reserves.balance)
            .map_err(generic_err)?
            .start();
        let pool_total_sol_value = SyncSolVal {
            pool_total: *total_sol_value,
            lst_old: old_sol_val,
            lst_new: new_sol_val,
        }
        .exec();

        let AddLiqQuote(inf1_core::quote::Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            ..
        }) = quote_add_liq(AddLiqQuoteArgs {
            amt: *amt,
            lp_token_supply,
            pool_total_sol_value,
            lp_protocol_fee_bps: *lp_protocol_fee_bps,
            inp_mint: *inp_mint,
            lp_mint: *lp_token_mint,
            inp_calc,
            pricing: pricing.to_price_lp_tokens_to_mint(),
        })?;
        Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            fee_mint: FeeMint::Inp,
            mints: *mints,
        }
    } else if inp_mint == lp_token_mint {
        // remove liquidity
        let lp_token_supply = lp_token_supply.ok_or_else(|| missing_acc_err(lp_token_mint))?;
        let (out_calc, out_reserves) = lsts
            .get(out_mint)
            .and_then(|(c, r)| {
                let calc = c.as_sol_val_calc()?;
                let reserves = r.as_ref()?;
                Some((calc, reserves))
            })
            .ok_or_else(|| missing_svc_data(out_mint))?;

        // need to perform a manual SyncSolValue of out mint first
        // in case pool_total_sol_value is stale
        let lst_state_list = inf.lst_state_list();
        let (
            _i,
            LstState {
                sol_value: old_sol_val,
                ..
            },
        ) = try_find_lst_state(lst_state_list, out_mint)?;
        let new_sol_val = *out_calc
            .lst_to_sol(out_reserves.balance)
            .map_err(generic_err)?
            .start();
        let pool_total_sol_value = SyncSolVal {
            pool_total: *total_sol_value,
            lst_old: old_sol_val,
            lst_new: new_sol_val,
        }
        .exec();
        let pricing = pricing
            .to_price_lp_tokens_to_redeem()
            .ok_or_else(|| missing_acc_err(FlatFeeRedeemLpAccs::MAINNET.0.program_state()))?;

        let RemoveLiqQuote(inf1_core::quote::Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            ..
        }) = quote_remove_liq(RemoveLiqQuoteArgs {
            amt: *amt,
            lp_token_supply,
            pool_total_sol_value,
            out_reserves: out_reserves.balance,
            lp_protocol_fee_bps: *lp_protocol_fee_bps,
            out_mint: *out_mint,
            lp_mint: *lp_token_mint,
            out_calc,
            pricing,
        })?;
        Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            fee_mint: FeeMint::Out,
            mints: *mints,
        }
    } else {
        // swap
        todo!()
    };
    Ok(quote)
}

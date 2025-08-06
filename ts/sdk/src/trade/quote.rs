use bs58_fixed_wasm::Bs58Array;
use inf1_core::{
    inf1_pp_core::traits::collection::{PriceExactInCol, PriceExactOutCol},
    inf1_svc_core::traits::SolValCalc,
    quote::swap::{exact_in::quote_exact_in, exact_out::quote_exact_out, SwapQuote, SwapQuoteArgs},
    sync::SyncSolVal,
};
use inf1_svc_ag_core::calc::SvcCalcAg;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

#[allow(deprecated)]
use inf1_core::{
    inf1_pp_core::traits::deprecated::{PriceLpTokensToMintCol, PriceLpTokensToRedeemCol},
    quote::liquidity::{
        add::{quote_add_liq, AddLiqQuote, AddLiqQuoteArgs},
        remove::{quote_remove_liq, RemoveLiqQuote, RemoveLiqQuoteArgs},
    },
};

use crate::{
    err::{missing_svc_data_err, InfError},
    missing_acc_err,
    sol_val_calc::Calc,
    trade::{Pair, PkPair},
    utils::try_find_lst_state,
    Inf, Reserves,
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

/// @throws
#[wasm_bindgen(js_name = quoteTradeExactIn)]
pub fn quote_trade_exact_in(
    inf: &mut Inf,
    QuoteArgs { amt, mints }: &QuoteArgs,
) -> Result<Quote, InfError> {
    let PkPair(Pair {
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    }) = mints;
    let lp_token_mint = inf.pool.lp_token_mint;
    let lp_token_supply = inf.lp_token_supply;
    let total_sol_value = inf.pool.total_sol_value;
    let lp_protocol_fee_bps = inf.pool.lp_protocol_fee_bps;
    let trading_protocol_fee_bps = inf.pool.trading_protocol_fee_bps;

    let quote = if *out_mint == lp_token_mint {
        // add liquidity
        #[allow(deprecated)]
        // TODO: unwrap() because currently Infallible, will not be the case with Ag
        let pricing = inf.pricing.0.price_lp_tokens_to_mint_for(inp_mint).unwrap();
        let lp_token_supply = lp_token_supply.ok_or_else(|| missing_acc_err(&lp_token_mint))?;
        let (_i, inp_lst_state) = try_find_lst_state(inf.lst_state_list(), inp_mint)?;
        let (inp_calc, inp_reserves) = inf
            .try_get_or_init_lst(&inp_lst_state)
            .and_then(|(c, r)| to_calc_ag_reserves_balance(inp_mint, c, r))?;
        // need to perform a manual SyncSolValue of inp mint first
        // in case pool_total_sol_value is stale
        let old_sol_val = inp_lst_state.sol_value;
        let new_sol_val_range = inp_calc.lst_to_sol(inp_reserves.balance)?;
        let new_sol_val = new_sol_val_range.start();
        let pool_total_sol_value = SyncSolVal {
            pool_total: total_sol_value,
            lst_old: old_sol_val,
            lst_new: *new_sol_val,
        }
        .exec();

        #[allow(deprecated)]
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
            lp_protocol_fee_bps,
            inp_mint: *inp_mint,
            lp_mint: lp_token_mint,
            inp_calc,
            pricing,
        })?;
        Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            fee_mint: FeeMint::Inp,
            mints: *mints,
        }
    } else if *inp_mint == lp_token_mint {
        // remove liquidity
        #[allow(deprecated)]
        let pricing = inf.pricing.0.price_lp_tokens_to_redeem_for(out_mint)?;
        let lp_token_supply = lp_token_supply.ok_or_else(|| missing_acc_err(&lp_token_mint))?;
        let (_i, out_lst_state) = try_find_lst_state(inf.lst_state_list(), out_mint)?;
        let (out_calc, out_reserves) = inf
            .try_get_or_init_lst(&out_lst_state)
            .and_then(|(c, r)| to_calc_ag_reserves_balance(out_mint, c, r))?;

        // need to perform a manual SyncSolValue of out mint first
        // in case pool_total_sol_value is stale
        let old_sol_val = out_lst_state.sol_value;
        let new_sol_val = *out_calc.lst_to_sol(out_reserves.balance)?.start();
        let pool_total_sol_value = SyncSolVal {
            pool_total: total_sol_value,
            lst_old: old_sol_val,
            lst_new: new_sol_val,
        }
        .exec();

        #[allow(deprecated)]
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
            lp_protocol_fee_bps,
            out_mint: *out_mint,
            lp_mint: lp_token_mint,
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
        let pricing = inf.pricing.0.price_exact_in_for(&Pair {
            inp: inp_mint,
            out: out_mint,
        })?;
        let [inp_res, out_res]: [Result<_, InfError>; 2] = [inp_mint, out_mint].map(|mint| {
            let (_i, lst_state) = try_find_lst_state(inf.lst_state_list(), mint)?;
            inf.try_get_or_init_lst(&lst_state)
                .and_then(|(c, r)| to_calc_ag_reserves_balance(out_mint, c, r))
                .map(|(c, r)| (*c, *r))
        });
        let inp_data = inp_res?;
        let out_data = out_res?;

        let (inp_calc, _) = inp_data;
        let (out_calc, out_reserves) = out_data;

        let SwapQuote(inf1_core::quote::Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            ..
        }) = quote_exact_in(SwapQuoteArgs {
            amt: *amt,
            inp_mint: *inp_mint,
            out_mint: *out_mint,
            pricing,
            out_reserves: out_reserves.balance,
            trading_protocol_fee_bps,
            inp_calc,
            out_calc,
        })?;
        Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            fee_mint: FeeMint::Out,
            mints: *mints,
        }
    };
    Ok(quote)
}

/// @throws
#[wasm_bindgen(js_name = quoteTradeExactOut)]
pub fn quote_trade_exact_out(
    inf: &mut Inf,
    QuoteArgs { amt, mints }: &QuoteArgs,
) -> Result<Quote, InfError> {
    // only SwapExactOut is supported for exact out
    // a lot of repeated code with SwapExactIn here,
    // but keeping them for now to allow for decoupled evolution

    let PkPair(Pair {
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    }) = mints;

    let trading_protocol_fee_bps = inf.pool.trading_protocol_fee_bps;

    let pricing = inf.pricing.0.price_exact_out_for(&Pair {
        inp: inp_mint,
        out: out_mint,
    })?;

    let [inp_res, out_res]: [Result<_, InfError>; 2] = [inp_mint, out_mint].map(|mint| {
        let (_i, lst_state) = try_find_lst_state(inf.lst_state_list(), mint)?;
        inf.try_get_or_init_lst(&lst_state)
            .and_then(|(c, r)| to_calc_ag_reserves_balance(out_mint, c, r))
            .map(|(c, r)| (*c, *r))
    });
    let inp_data = inp_res?;
    let out_data = out_res?;

    let (inp_calc, _) = inp_data;
    let (out_calc, out_reserves) = out_data;

    let SwapQuote(inf1_core::quote::Quote {
        inp,
        out,
        lp_fee,
        protocol_fee,
        ..
    }) = quote_exact_out(SwapQuoteArgs {
        amt: *amt,
        inp_mint: *inp_mint,
        out_mint: *out_mint,
        pricing,
        out_reserves: out_reserves.balance,
        trading_protocol_fee_bps,
        inp_calc,
        out_calc,
    })?;
    Ok(Quote {
        inp,
        out,
        lp_fee,
        protocol_fee,
        fee_mint: FeeMint::Out,
        mints: *mints,
    })
}

fn to_calc_ag_reserves_balance<'a>(
    mint: &[u8; 32],
    calc: &'a Calc,
    reserves: &'a Option<Reserves>,
) -> Result<(&'a SvcCalcAg, &'a Reserves), InfError> {
    calc.as_sol_val_calc()
        .and_then(|calc| Some((calc, reserves.as_ref()?)))
        .ok_or_else(|| missing_svc_data_err(mint))
}

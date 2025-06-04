use bs58_fixed_wasm::Bs58Array;
use inf1_core::{
    inf1_ctl_core::accounts::pool_state::PoolState,
    quote::liquidity::add::{quote_add_liq, AddLiqQuote, AddLiqQuoteArgs},
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{err::missing_svc_data, missing_acc_err, trade::PkPair, InfHandle};

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
    InfHandle {
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
    }: &InfHandle,
    QuoteArgs { amt, mints }: &QuoteArgs,
) -> Result<Quote, JsError> {
    let PkPair {
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    } = mints;
    let quote = if out_mint == lp_token_mint {
        // add liquidity
        let lp_token_supply = lp_token_supply.ok_or_else(|| missing_acc_err(lp_token_mint))?;
        let inp_calc = lsts
            .get(inp_mint)
            .and_then(|(c, _)| c.as_sol_val_calc())
            .ok_or_else(|| missing_svc_data(inp_mint))?;
        let AddLiqQuote(inf1_core::quote::Quote {
            inp,
            out,
            lp_fee,
            protocol_fee,
            ..
        }) = quote_add_liq(AddLiqQuoteArgs {
            amt: *amt,
            lp_token_supply,
            pool_total_sol_value: *total_sol_value,
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
        todo!()
    } else {
        // swap
        todo!()
    };
    Ok(quote)
}

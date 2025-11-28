use bs58_fixed_wasm::Bs58Array;
use inf1_std::quote::swap::SwapQuote;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{err::InfError, interface::PkPair, trade::Pair, Inf};

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
    let PkPair {
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    } = mints;
    let SwapQuote(inf1_std::quote::Quote {
        inp,
        out,
        lp_fee,
        protocol_fee,
        ..
    }) = inf.0.quote_exact_in_mut(
        &Pair {
            inp: inp_mint,
            out: out_mint,
        },
        *amt,
    )?;
    Ok(Quote {
        inp,
        out,
        lp_fee,
        protocol_fee,
        fee_mint: FeeMint::Out,
        mints: *mints,
    })
}

/// @throws
#[wasm_bindgen(js_name = quoteTradeExactOut)]
pub fn quote_trade_exact_out(
    inf: &mut Inf,
    QuoteArgs { amt, mints }: &QuoteArgs,
) -> Result<Quote, InfError> {
    // A lot of repeated code with SwapExactIn here,
    // but keeping them for now to allow for decoupled evolution

    let PkPair {
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    } = mints;

    let SwapQuote(inf1_std::quote::Quote {
        inp,
        out,
        lp_fee,
        protocol_fee,
        ..
    }) = inf.0.quote_exact_out_mut(
        &Pair {
            inp: inp_mint,
            out: out_mint,
        },
        *amt,
    )?;
    Ok(Quote {
        inp,
        out,
        lp_fee,
        protocol_fee,
        fee_mint: FeeMint::Out,
        mints: *mints,
    })
}

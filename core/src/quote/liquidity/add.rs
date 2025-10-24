#![deprecated(
    since = "0.2.0",
    note = "Use SwapExactIn/Out with out_mint=LP token (INF) instead"
)]

use core::{error::Error, fmt::Display};

use inf1_pp_core::instructions::IxArgs;
use inf1_svc_core::traits::SolValCalc;
use sanctum_fee_ratio::ratio::{Floor, Ratio};

#[allow(deprecated)]
use inf1_pp_core::traits::deprecated::PriceLpTokensToMint;

use crate::quote::{liquidity::lp_protocol_fee, Quote};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AddLiqQuoteArgs<I, P> {
    pub amt: u64,

    pub lp_token_supply: u64,

    /// This should be the updated value after
    /// syncing SOL value of the pools' inp reserves.
    /// The value currently in account data may be stale.
    pub pool_total_sol_value: u64,

    /// Read from PoolState
    pub lp_protocol_fee_bps: u16,

    pub inp_mint: [u8; 32],

    pub lp_mint: [u8; 32],

    pub inp_calc: I,

    pub pricing: P,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AddLiqQuote(pub Quote);

impl AddLiqQuote {
    #[inline]
    pub const fn fee_mint(&self) -> &[u8; 32] {
        &self.0.inp_mint
    }
}

pub type AddLiqQuoteResult<I, P> = Result<AddLiqQuote, AddLiqQuoteErr<I, P>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddLiqQuoteErr<I, P> {
    InpCalc(I),
    Overflow,
    Pricing(P),
    ZeroValue,
}

impl<S: Display, P: Display> Display for AddLiqQuoteErr<S, P> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Overflow => f.write_str("arithmetic overflow"),
            Self::Pricing(e) => e.fmt(f),
            Self::InpCalc(e) => e.fmt(f),
            Self::ZeroValue => f.write_str("zero value"),
        }
    }
}

// fully qualify core::fmt::Debug instead of importing so that .fmt() doesnt clash with Display
impl<I: core::fmt::Debug + Display, P: core::fmt::Debug + Display> Error for AddLiqQuoteErr<I, P> {}

#[allow(deprecated)]
pub fn quote_add_liq<I: SolValCalc, P: PriceLpTokensToMint>(
    AddLiqQuoteArgs {
        amt,
        lp_token_supply,
        pool_total_sol_value,
        lp_protocol_fee_bps,
        inp_mint,
        lp_mint,
        inp_calc,
        pricing,
    }: AddLiqQuoteArgs<I, P>,
) -> AddLiqQuoteResult<I::Error, P::Error> {
    let amt_sol_val = *inp_calc
        .lst_to_sol(amt)
        .map_err(AddLiqQuoteErr::InpCalc)?
        .start();

    let amt_sol_val_after_fees = pricing
        .price_lp_tokens_to_mint(IxArgs {
            amt,
            sol_value: amt_sol_val,
        })
        .map_err(AddLiqQuoteErr::Pricing)?;
    let fees_sol_val = amt_sol_val.saturating_sub(amt_sol_val_after_fees);
    let protocol_fee = lp_protocol_fee(lp_protocol_fee_bps).ok_or(AddLiqQuoteErr::Overflow)?;
    let aft_pf = protocol_fee
        .apply(fees_sol_val)
        .ok_or(AddLiqQuoteErr::Overflow)?;
    // NB: lp_fee is just an estimate because no tokens are actually transferred
    let [Some(protocol_fee), Some(lp_fee)] = [aft_pf.fee(), aft_pf.rem()].map(|sol_val| {
        Floor(Ratio {
            n: amt,
            d: amt_sol_val,
        })
        .apply(sol_val)
    }) else {
        return Err(AddLiqQuoteErr::Overflow);
    };

    let to_mint = if lp_token_supply == 0 {
        // edge-case: if LP supply 0,
        // make it s.t. lp_token:sol_value 1:1 exchange rate
        pool_total_sol_value
            .checked_add(amt_sol_val_after_fees)
            .ok_or(AddLiqQuoteErr::Overflow)?
    } else if pool_total_sol_value == 0 {
        // edge-case: if LP supply nonzero but pool sol value 0,
        // mint amount == final_sol_value_to_add.
        // This dilutes the LPer but ensures pool can still function.
        // Should never happen.
        amt_sol_val_after_fees
    } else {
        Floor(Ratio {
            n: lp_token_supply,
            d: pool_total_sol_value,
        })
        .apply(amt_sol_val_after_fees)
        .ok_or(AddLiqQuoteErr::Overflow)?
    };

    if to_mint == 0 || protocol_fee >= amt {
        return Err(AddLiqQuoteErr::ZeroValue);
    }

    Ok(AddLiqQuote(Quote {
        inp: amt,
        out: to_mint,
        lp_fee,
        protocol_fee,
        inp_mint,
        out_mint: lp_mint,
    }))
}

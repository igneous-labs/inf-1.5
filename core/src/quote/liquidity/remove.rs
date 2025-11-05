#![deprecated(
    since = "0.2.0",
    note = "Use SwapExactIn/Out with inp_mint=LP token (INF) instead"
)]

use core::{error::Error, fmt::Display};

use inf1_pp_core::instructions::IxArgs;
use inf1_svc_core::traits::SolValCalc;
use sanctum_fee_ratio::ratio::{Floor, Ratio};

#[allow(deprecated)]
use inf1_pp_core::traits::deprecated::PriceLpTokensToRedeem;

use crate::{err::NotEnoughLiquidityErr, quote::Quote, typedefs::fee_bps::fee_bps};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemoveLiqQuoteArgs<O, P> {
    /// Amount of LP tokens to burn to redeem
    pub amt: u64,

    pub lp_token_supply: u64,

    /// This should be the updated value after
    /// syncing SOL value of the pools' out reserves.
    /// The value currently in account data may be stale.
    pub pool_total_sol_value: u64,

    pub out_reserves: u64,

    /// Read from PoolState
    pub lp_protocol_fee_bps: u16,

    pub out_mint: [u8; 32],

    pub lp_mint: [u8; 32],

    pub out_calc: O,

    pub pricing: P,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RemoveLiqQuote(pub Quote);

impl RemoveLiqQuote {
    #[inline]
    pub const fn fee_mint(&self) -> &[u8; 32] {
        &self.0.out_mint
    }
}

pub type RemoveLiqQuoteResult<O, P> = Result<RemoveLiqQuote, RemoveLiqQuoteErr<O, P>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoveLiqQuoteErr<O, P> {
    NotEnougLiquidity(NotEnoughLiquidityErr),
    OutCalc(O),
    Overflow,
    Pricing(P),
    ZeroValue,
}

impl<O: Display, P: Display> Display for RemoveLiqQuoteErr<O, P> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotEnougLiquidity(e) => e.fmt(f),
            Self::OutCalc(e) => e.fmt(f),
            Self::Overflow => f.write_str("arithmetic overflow"),
            Self::Pricing(e) => e.fmt(f),
            Self::ZeroValue => f.write_str("zero value"),
        }
    }
}

// fully qualify core::fmt::Debug instead of importing so that .fmt() doesnt clash with Display
impl<I: core::fmt::Debug + Display, P: core::fmt::Debug + Display> Error
    for RemoveLiqQuoteErr<I, P>
{
}

#[allow(deprecated)]
pub fn quote_remove_liq<O: SolValCalc, P: PriceLpTokensToRedeem>(
    RemoveLiqQuoteArgs {
        amt,
        lp_token_supply,
        pool_total_sol_value,
        out_reserves,
        lp_protocol_fee_bps,
        out_mint,
        lp_mint,
        out_calc,
        pricing,
    }: RemoveLiqQuoteArgs<O, P>,
) -> RemoveLiqQuoteResult<O::Error, P::Error> {
    let lp_tokens_sol_value = if pool_total_sol_value == 0 || lp_token_supply == 0 {
        0
    } else {
        Floor(Ratio {
            n: pool_total_sol_value,
            d: lp_token_supply,
        })
        .apply(amt)
        .ok_or(RemoveLiqQuoteErr::Overflow)?
    };
    let lp_tokens_sol_value_after_fees = pricing
        .price_lp_tokens_to_redeem(IxArgs {
            amt,
            sol_value: lp_tokens_sol_value,
        })
        .map_err(RemoveLiqQuoteErr::Pricing)?;
    let to_user_lst_amount = *out_calc
        .sol_to_lst(lp_tokens_sol_value_after_fees)
        .map_err(RemoveLiqQuoteErr::OutCalc)?
        .start();
    // If user lst_amount to return is greater than the balance of the lst of the reserve
    // there won't be enough liq to redeem the lst
    if to_user_lst_amount > out_reserves {
        return Err(RemoveLiqQuoteErr::NotEnougLiquidity(
            NotEnoughLiquidityErr {
                required: to_user_lst_amount,
                available: out_reserves,
            },
        ));
    }
    let lp_fees_sol_value = lp_tokens_sol_value.saturating_sub(lp_tokens_sol_value_after_fees);
    let protocol_fee = fee_bps(lp_protocol_fee_bps).ok_or(RemoveLiqQuoteErr::Overflow)?;
    let aft_pf = protocol_fee
        .apply(lp_fees_sol_value)
        .ok_or(RemoveLiqQuoteErr::Overflow)?;
    // NB: lp_fee is just an estimate because no tokens are actually transferred
    let [Some(protocol_fee), Some(lp_fee)] = [aft_pf.fee(), aft_pf.rem()].map(|sol_val| {
        Floor(Ratio {
            n: to_user_lst_amount,
            d: lp_tokens_sol_value_after_fees,
        })
        .apply(sol_val)
    }) else {
        return Err(RemoveLiqQuoteErr::Overflow);
    };

    Ok(RemoveLiqQuote(Quote {
        inp: amt,
        out: to_user_lst_amount,
        lp_fee,
        protocol_fee,
        inp_mint: lp_mint,
        out_mint,
    }))
}

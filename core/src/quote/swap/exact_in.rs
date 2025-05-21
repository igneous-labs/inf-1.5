use inf1_pp_core::{instructions::IxArgs, traits::PriceExactIn};
use inf1_svc_core::traits::SolValCalc;
use sanctum_fee_ratio::ratio::{Floor, Ratio};

use crate::{
    err::NotEnoughLiquidityErr,
    quote::{swap::trading_protocol_fee, Quote},
};

use super::{err::SwapQuoteErr, SwapQuote, SwapQuoteArgs, SwapQuoteResult};

pub fn quote_exact_in<S: SolValCalc, D: SolValCalc, P: PriceExactIn>(
    SwapQuoteArgs {
        amt,
        out_reserves,
        trading_protocol_fee_bps,
        src_calc,
        dst_calc,
        pricing,
        inp_mint,
        out_mint,
    }: SwapQuoteArgs<S, D, P>,
) -> SwapQuoteResult<S::Error, D::Error, P::Error> {
    let in_sol_val = *src_calc
        .lst_to_sol(amt)
        .map_err(SwapQuoteErr::SrcCalc)?
        .start();
    if in_sol_val == 0 {
        return Err(SwapQuoteErr::ZeroValue);
    }

    let out_sol_val = pricing
        .price_exact_in(IxArgs {
            amt,
            sol_value: in_sol_val,
        })
        .map_err(SwapQuoteErr::Pricing)?;

    let dst_lst_out = *dst_calc
        .sol_to_lst(out_sol_val)
        .map_err(SwapQuoteErr::DstCalc)?
        .start();
    if dst_lst_out == 0 {
        return Err(SwapQuoteErr::ZeroValue);
    }

    let protocol_fee =
        trading_protocol_fee(trading_protocol_fee_bps).ok_or(SwapQuoteErr::Overflow)?;
    let fees_sol_val = in_sol_val.saturating_sub(out_sol_val);
    let aft_pf = protocol_fee
        .apply(fees_sol_val)
        .ok_or(SwapQuoteErr::Overflow)?;

    // NB: lp_fee is just an estimate because no tokens are actually transferred
    let [Some(protocol_fee), Some(lp_fee)] = [aft_pf.fee(), aft_pf.rem()].map(|sol_val| {
        Floor(Ratio {
            n: dst_lst_out,
            d: out_sol_val,
        })
        .apply(sol_val)
    }) else {
        return Err(SwapQuoteErr::Overflow);
    };

    let total_dst_lst_out = protocol_fee
        .checked_add(dst_lst_out)
        .ok_or(SwapQuoteErr::Overflow)?;
    if out_reserves < total_dst_lst_out {
        return Err(SwapQuoteErr::NotEnougLiquidity(NotEnoughLiquidityErr {
            required: total_dst_lst_out,
            available: out_reserves,
        }));
    }

    Ok(SwapQuote(Quote {
        inp: amt,
        out: dst_lst_out,
        lp_fee,
        protocol_fee,
        inp_mint,
        out_mint,
    }))
}

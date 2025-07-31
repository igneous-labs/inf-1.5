use inf1_pp_core::{instructions::IxArgs, traits::main::PriceExactOut};
use inf1_svc_core::traits::SolValCalc;
use sanctum_fee_ratio::ratio::{Floor, Ratio};

use crate::{
    err::NotEnoughLiquidityErr,
    quote::{
        swap::{err::SwapQuoteErr, trading_protocol_fee},
        Quote,
    },
};

use super::{SwapQuote, SwapQuoteArgs, SwapQuoteResult};

pub fn quote_exact_out<I: SolValCalc, O: SolValCalc, P: PriceExactOut>(
    SwapQuoteArgs {
        amt,
        out_reserves,
        trading_protocol_fee_bps,
        inp_calc,
        out_calc,
        pricing,
        inp_mint,
        out_mint,
    }: SwapQuoteArgs<I, O, P>,
) -> SwapQuoteResult<I::Error, O::Error, P::Error> {
    let out_sol_val = *out_calc
        .lst_to_sol(amt)
        .map_err(SwapQuoteErr::OutCalc)?
        .end();
    if out_sol_val == 0 {
        return Err(SwapQuoteErr::ZeroValue);
    }

    let in_sol_val = pricing
        .price_exact_out(IxArgs {
            amt,
            sol_value: out_sol_val,
        })
        .map_err(SwapQuoteErr::Pricing)?;
    let inp = *inp_calc
        .sol_to_lst(in_sol_val)
        .map_err(SwapQuoteErr::InpCalc)?
        .end();
    if inp == 0 {
        return Err(SwapQuoteErr::ZeroValue);
    }

    let fees_sol_val = in_sol_val.saturating_sub(out_sol_val);
    let protocol_fee =
        trading_protocol_fee(trading_protocol_fee_bps).ok_or(SwapQuoteErr::Overflow)?;
    let aft_pf = protocol_fee
        .apply(fees_sol_val)
        .ok_or(SwapQuoteErr::Overflow)?;

    // NB: lp_fee is just an estimate because no tokens are actually transferred
    let [Some(protocol_fee), Some(lp_fee)] = [aft_pf.fee(), aft_pf.rem()].map(|sol_val| {
        Floor(Ratio {
            n: amt,
            d: out_sol_val,
        })
        .apply(sol_val)
    }) else {
        return Err(SwapQuoteErr::Overflow);
    };

    let total_out_lst_out = protocol_fee
        .checked_add(amt)
        .ok_or(SwapQuoteErr::Overflow)?;
    if out_reserves < total_out_lst_out {
        return Err(SwapQuoteErr::NotEnoughLiquidity(NotEnoughLiquidityErr {
            required: total_out_lst_out,
            available: out_reserves,
        }));
    }

    Ok(SwapQuote(Quote {
        inp,
        out: amt,
        lp_fee,
        protocol_fee,
        inp_mint,
        out_mint,
    }))
}

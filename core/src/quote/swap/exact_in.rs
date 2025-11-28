use inf1_pp_core::{instructions::IxArgs, traits::main::PriceExactIn};
use inf1_svc_core::traits::SolValCalc;

use crate::{err::NotEnoughLiquidityErr, quote::Quote};

use super::{err::QuoteErr, QuoteArgs, QuoteResult};

pub fn quote_exact_in<I: SolValCalc, O: SolValCalc, P: PriceExactIn>(
    QuoteArgs {
        amt,
        out_reserves,
        inp_calc,
        out_calc,
        pricing,
        inp_mint,
        out_mint,
    }: &QuoteArgs<I, O, P>,
) -> QuoteResult<I::Error, O::Error, P::Error> {
    let inp_sol_val = *inp_calc
        .lst_to_sol(*amt)
        .map_err(QuoteErr::InpCalc)?
        .start();
    if inp_sol_val == 0 {
        return Err(QuoteErr::ZeroValue);
    }

    let out_sol_val = pricing
        .price_exact_in(IxArgs {
            amt: *amt,
            sol_value: inp_sol_val,
        })
        .map_err(QuoteErr::Pricing)?;

    let out = *out_calc
        .sol_to_lst(out_sol_val)
        .map_err(QuoteErr::OutCalc)?
        .start();
    if out == 0 {
        return Err(QuoteErr::ZeroValue);
    }

    let fee_sol_val = inp_sol_val
        .checked_sub(out_sol_val)
        .ok_or(QuoteErr::PoolLoss)?;

    if *out_reserves < out {
        return Err(QuoteErr::NotEnoughLiquidity(NotEnoughLiquidityErr {
            required: out,
            available: *out_reserves,
        }));
    }

    Ok(Quote {
        inp: *amt,
        out,
        fee: fee_sol_val,
        inp_mint: *inp_mint,
        out_mint: *out_mint,
    })
}

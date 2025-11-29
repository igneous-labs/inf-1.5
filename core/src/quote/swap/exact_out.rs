use inf1_pp_core::{instructions::IxArgs, traits::main::PriceExactOut};
use inf1_svc_core::traits::SolValCalc;

use crate::{
    err::NotEnoughLiquidityErr,
    quote::{swap::err::QuoteErr, Quote},
};

use super::{QuoteArgs, QuoteResult};

pub fn quote_exact_out<I: SolValCalc, O: SolValCalc, P: PriceExactOut>(
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
    let out_sol_val = *out_calc.lst_to_sol(*amt).map_err(QuoteErr::OutCalc)?.end();
    if out_sol_val == 0 {
        return Err(QuoteErr::ZeroValue);
    }

    let inp_sol_val = pricing
        .price_exact_out(IxArgs {
            amt: *amt,
            sol_value: out_sol_val,
        })
        .map_err(QuoteErr::Pricing)?;
    let inp = *inp_calc
        .sol_to_lst(inp_sol_val)
        .map_err(QuoteErr::InpCalc)?
        .end();
    if inp == 0 {
        return Err(QuoteErr::ZeroValue);
    }

    let fee_sol_val = inp_sol_val
        .checked_sub(out_sol_val)
        .ok_or(QuoteErr::PoolLoss)?;

    if out_reserves < amt {
        return Err(QuoteErr::NotEnoughLiquidity(NotEnoughLiquidityErr {
            required: *amt,
            available: *out_reserves,
        }));
    }

    Ok(Quote {
        inp,
        out: *amt,
        fee: fee_sol_val,
        inp_mint: *inp_mint,
        out_mint: *out_mint,
    })
}

use std::convert::Infallible;

use inf1_core::quote::swap::err::QuoteErr;
use inf1_ctl_jiminy::err::Inf1CtlErr;

#[inline]
pub fn quote_err_to_inf1_ctl_err(e: QuoteErr<Infallible, Infallible, Infallible>) -> Inf1CtlErr {
    match e {
        // Onchain interfaces have Infallible error since we call the CPI first and just wrap the result
        QuoteErr::InpCalc(_inflb) | QuoteErr::OutCalc(_inflb) | QuoteErr::Pricing(_inflb) => {
            unreachable!()
        }
        QuoteErr::InpDisabled => Inf1CtlErr::LstInputDisabled,
        QuoteErr::NotEnoughLiquidity(_) => Inf1CtlErr::NotEnoughLiquidity,
        QuoteErr::PoolLoss => Inf1CtlErr::PoolWouldLoseSolValue,
        QuoteErr::ZeroValue => Inf1CtlErr::ZeroValue,
    }
}

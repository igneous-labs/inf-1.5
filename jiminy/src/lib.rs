use core::fmt::Display;
use inf1_core::quote::swap::err::QuoteErr;
use inf1_ctl_jiminy::{err::Inf1CtlErr, program_err::Inf1CtlCustomProgErr};
use jiminy_log::sol_log;
use jiminy_program_error::ProgramError;

pub struct SwapQuoteProgErr<I, O, P>(pub QuoteErr<I, O, P>);

impl<
        I: Display + Into<ProgramError>,
        O: Display + Into<ProgramError>,
        P: Display + Into<ProgramError>,
    > From<SwapQuoteProgErr<I, O, P>> for ProgramError
{
    fn from(SwapQuoteProgErr(e): SwapQuoteProgErr<I, O, P>) -> Self {
        let msg = e.to_string();
        sol_log(&msg);
        match e {
            QuoteErr::InpCalc(e) => e.into(),
            QuoteErr::OutCalc(e) => e.into(),
            QuoteErr::PoolLoss => Inf1CtlCustomProgErr(Inf1CtlErr::MathError).into(),
            QuoteErr::NotEnoughLiquidity(_) => {
                Inf1CtlCustomProgErr(Inf1CtlErr::NotEnoughLiquidity).into()
            }
            QuoteErr::Pricing(e) => e.into(),
            QuoteErr::ZeroValue => Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into(),
        }
    }
}

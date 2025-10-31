use core::fmt::Display;
use inf1_core::quote::swap::err::SwapQuoteErr;
use inf1_ctl_jiminy::{err::Inf1CtlErr, program_err::Inf1CtlCustomProgErr};
use jiminy_log::sol_log;
use jiminy_program_error::ProgramError;

pub struct SwapQuoteProgErr<I, O, P>(pub SwapQuoteErr<I, O, P>);

impl<
        I: core::fmt::Debug + Display,
        O: core::fmt::Debug + Display,
        P: core::fmt::Debug + Display,
    > From<SwapQuoteProgErr<I, O, P>> for ProgramError
where
    ProgramError: From<I> + From<O> + From<P>,
{
    fn from(SwapQuoteProgErr(e): SwapQuoteProgErr<I, O, P>) -> Self {
        let msg = e.to_string();
        sol_log(&msg);
        match e {
            SwapQuoteErr::InpCalc(e) => e.into(),
            SwapQuoteErr::OutCalc(e) => e.into(),
            SwapQuoteErr::Overflow => Inf1CtlCustomProgErr(Inf1CtlErr::MathError).into(),
            SwapQuoteErr::NotEnoughLiquidity(_) => {
                Inf1CtlCustomProgErr(Inf1CtlErr::NotEnoughLiquidity).into()
            }
            SwapQuoteErr::Pricing(e) => e.into(),
            SwapQuoteErr::ZeroValue => Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into(),
        }
    }
}

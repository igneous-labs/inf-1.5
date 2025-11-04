use core::fmt::Display;
#[allow(deprecated)]
use inf1_core::quote::{liquidity::add::AddLiqQuoteErr, swap::err::SwapQuoteErr};
use inf1_ctl_jiminy::{err::Inf1CtlErr, program_err::Inf1CtlCustomProgErr};
use jiminy_log::sol_log;
use jiminy_program_error::ProgramError;

pub struct SwapQuoteProgErr<I, O, P>(pub SwapQuoteErr<I, O, P>);

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
#[allow(deprecated)]
pub struct AddLiqQuoteProgErr<I, P>(pub AddLiqQuoteErr<I, P>);

impl<I: Display + Into<ProgramError>, P: Display + Into<ProgramError>>
    From<AddLiqQuoteProgErr<I, P>> for ProgramError
{
    #[allow(deprecated)]
    fn from(AddLiqQuoteProgErr(e): AddLiqQuoteProgErr<I, P>) -> Self {
        let msg = e.to_string();
        sol_log(&msg);
        match e {
            AddLiqQuoteErr::Overflow => Inf1CtlCustomProgErr(Inf1CtlErr::MathError).into(),
            AddLiqQuoteErr::ZeroValue => Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into(),
            AddLiqQuoteErr::InpCalc(e) => e.into(),
            AddLiqQuoteErr::Pricing(e) => e.into(),
        }
    }
}

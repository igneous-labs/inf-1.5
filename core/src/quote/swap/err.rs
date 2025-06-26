use core::{error::Error, fmt::Display};

use crate::err::NotEnoughLiquidityErr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapQuoteErr<I, O, P> {
    InpCalc(I),
    OutCalc(O),
    Overflow,
    NotEnoughLiquidity(NotEnoughLiquidityErr),
    Pricing(P),
    ZeroValue,
}

impl<I: Display, O: Display, P: Display> Display for SwapQuoteErr<I, O, P> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutCalc(e) => e.fmt(f),
            Self::Overflow => f.write_str("arithmetic overflow"),
            Self::NotEnoughLiquidity(e) => e.fmt(f),
            Self::Pricing(e) => e.fmt(f),
            Self::InpCalc(e) => e.fmt(f),
            Self::ZeroValue => f.write_str("zero value"),
        }
    }
}

// fully qualify core::fmt::Debug instead of importing so that .fmt() doesnt clash with Display
impl<
        I: core::fmt::Debug + Display,
        O: core::fmt::Debug + Display,
        P: core::fmt::Debug + Display,
    > Error for SwapQuoteErr<I, O, P>
{
}

impl<I, O, P> From<NotEnoughLiquidityErr> for SwapQuoteErr<I, O, P> {
    #[inline]
    fn from(value: NotEnoughLiquidityErr) -> Self {
        Self::NotEnoughLiquidity(value)
    }
}

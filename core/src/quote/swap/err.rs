use core::{error::Error, fmt::Display};

use crate::err::NotEnoughLiquidityErr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapQuoteErr<S, D, P> {
    DstCalc(D),
    Overflow,
    NotEnougLiquidity(NotEnoughLiquidityErr),
    Pricing(P),
    SrcCalc(S),
    ZeroValue,
}

impl<S: Display, D: Display, P: Display> Display for SwapQuoteErr<S, D, P> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DstCalc(e) => e.fmt(f),
            Self::Overflow => f.write_str("arithmetic overflow"),
            Self::NotEnougLiquidity(e) => e.fmt(f),
            Self::Pricing(e) => e.fmt(f),
            Self::SrcCalc(e) => e.fmt(f),
            Self::ZeroValue => f.write_str("zero value"),
        }
    }
}

// fully qualify core::fmt::Debug instead of importing so that .fmt() doesnt clash with Display
impl<
        S: core::fmt::Debug + Display,
        D: core::fmt::Debug + Display,
        P: core::fmt::Debug + Display,
    > Error for SwapQuoteErr<S, D, P>
{
}

impl<S, D, P> From<NotEnoughLiquidityErr> for SwapQuoteErr<S, D, P> {
    #[inline]
    fn from(value: NotEnoughLiquidityErr) -> Self {
        Self::NotEnougLiquidity(value)
    }
}

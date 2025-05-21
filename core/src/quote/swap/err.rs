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

impl<S, D, P> From<NotEnoughLiquidityErr> for SwapQuoteErr<S, D, P> {
    #[inline]
    fn from(value: NotEnoughLiquidityErr) -> Self {
        Self::NotEnougLiquidity(value)
    }
}

pub mod instruction;
pub mod quote;
pub mod update;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trade<AddLiquidity, RemoveLiquidity, SwapExactIn, SwapExactOut> {
    AddLiquidity(AddLiquidity),
    RemoveLiquidity(RemoveLiquidity),
    SwapExactIn(SwapExactIn),
    SwapExactOut(SwapExactOut),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TradeLimitTy {
    ExactIn,
    ExactOut,
}

// Note about `_mut()` methods vs non-`_mut`:
// Former requires `&mut self` access in order to call `try_get_or_init_lst_svc_mut`
// to lazily initialize LSTs as required.
// Latter functions with just `&self`, and fails if the LST's sol value calculator
// vars were not initialized.

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

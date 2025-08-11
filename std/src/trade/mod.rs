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

// Iterator blanket
impl<
        T,
        AddLiquidity: Iterator<Item = T>,
        RemoveLiquidity: Iterator<Item = T>,
        SwapExactIn: Iterator<Item = T>,
        SwapExactOut: Iterator<Item = T>,
    > Iterator for Trade<AddLiquidity, RemoveLiquidity, SwapExactIn, SwapExactOut>
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::AddLiquidity(c) => c.next(),
            Self::RemoveLiquidity(c) => c.next(),
            Self::SwapExactIn(c) => c.next(),
            Self::SwapExactOut(c) => c.next(),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::AddLiquidity(c) => c.fold(init, f),
            Self::RemoveLiquidity(c) => c.fold(init, f),
            Self::SwapExactIn(c) => c.fold(init, f),
            Self::SwapExactOut(c) => c.fold(init, f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TradeLimitTy {
    ExactIn,
    ExactOut,
}

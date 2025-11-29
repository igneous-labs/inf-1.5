// Note about `_mut()` methods vs non-`_mut`:
// Former requires `&mut self` access in order to call `try_get_or_init_lst_svc_mut`
// to lazily initialize LSTs as required.
// Latter functions with just `&self`, and fails if the LST's sol value calculator
// vars were not initialized.

pub mod instruction;
pub mod quote;
pub mod update;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trade<ExactIn, ExactOut> {
    ExactIn(ExactIn),
    ExactOut(ExactOut),
}

// Iterator blanket
impl<T, ExactIn: Iterator<Item = T>, ExactOut: Iterator<Item = T>> Iterator
    for Trade<ExactIn, ExactOut>
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::ExactIn(c) => c.next(),
            Self::ExactOut(c) => c.next(),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::ExactIn(c) => c.fold(init, f),
            Self::ExactOut(c) => c.fold(init, f),
        }
    }
}

pub type TradeLimitTy = Trade<(), ()>;

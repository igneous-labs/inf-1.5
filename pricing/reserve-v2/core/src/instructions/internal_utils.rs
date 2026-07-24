/// Chain Slice Iter for 'a T generic
///
/// We want to use `Chain<slice...>` over stuff like `Flatten<array...>` because
/// the former impls TrustedLen while the latter does not. This also somewhat
/// standardizes iterator types across all instructions
///
/// Only way to make it work with decl macros is to
/// tt-munch one token each time. This is why we have `csi_at!(@ @)`
/// instead of `csi_at!(2)`
macro_rules! csi_at {
    // Recursive-case: add a Chain
    (@ $($tail:tt)*) => {
        core::iter::Chain<csi_at!($($tail)*), core::slice::Iter<'a, T>>
    };

    // Base-case: output single slice::Iter
    () => {
        core::slice::Iter<'a, T>
    };
}
pub(crate) use csi_at;

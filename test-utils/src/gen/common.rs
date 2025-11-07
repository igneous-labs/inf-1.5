use proptest::prelude::*;

macro_rules! int_strat {
    ($f:ident, $I:ty) => {
        pub fn $f(ovride: Option<BoxedStrategy<$I>>) -> BoxedStrategy<$I> {
            ovride.unwrap_or_else(|| (0..=<$I>::MAX).boxed())
        }
    };
}
int_strat!(u8_strat, u8);
int_strat!(u16_strat, u16);
int_strat!(u64_strat, u64);

pub fn bool_strat(ovride: Option<BoxedStrategy<bool>>) -> BoxedStrategy<bool> {
    ovride.unwrap_or_else(|| any::<bool>().boxed())
}

pub fn pk_strat(ovrride: Option<BoxedStrategy<[u8; 32]>>) -> BoxedStrategy<[u8; 32]> {
    ovrride.unwrap_or_else(|| any::<[u8; 32]>().boxed())
}

/// Converts a `Option<Strategy>` to `Strategy<Option>`,
/// returning `Just(Some(strat_output))` if `Some`, `Just(None)` if `None`
pub fn opt_transpose_strat<T: core::fmt::Debug + Clone + 'static>(
    opt: Option<BoxedStrategy<T>>,
) -> BoxedStrategy<Option<T>> {
    opt.map_or_else(|| Just(None).boxed(), |s| s.prop_map(Some).boxed())
}

/// Strategy that generates indexes that are out of bounds for a vec of given len
///
/// upper bound of u32::MAX instead of usize::MAX
pub fn idx_oob(list_len: usize) -> impl Strategy<Value = usize> {
    list_len..=u32::MAX as usize
}

/// Returns 2 distinct valid indexes for a vec of given len
pub fn distinct_idxs(list_len: usize) -> impl Strategy<Value = (usize, usize)> {
    (0..list_len, 0..list_len).prop_filter("", |(x, y)| x != y)
}

pub fn list_sample_flat_map<T: Clone + core::fmt::Debug>(
    l: Vec<T>,
) -> impl Strategy<Value = (usize, T, Vec<T>)> {
    (0..l.len(), Just(l)).prop_map(|(i, l)| (i, l[i].clone(), l))
}

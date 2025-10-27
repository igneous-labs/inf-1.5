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

/// Converts a Option<Strategy> to Strategy<Option>,
/// returining Just(Some(strat_output)) or Just(None)
pub fn opt_transpose_strat<T: core::fmt::Debug + Clone + 'static>(
    opt: Option<BoxedStrategy<T>>,
) -> BoxedStrategy<Option<T>> {
    opt.map_or_else(|| Just(None).boxed(), |s| s.prop_map(Some).boxed())
}

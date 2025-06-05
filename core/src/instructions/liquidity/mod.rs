use core::{
    iter::{once, Chain, Once},
    slice,
};

use inf1_ctl_core::instructions::liquidity as inf1_ctl_core_liquidity;
use inf1_svc_core::traits::SolValCalcAccs;

pub mod add;
pub mod remove;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxAccs<T, I, C, P> {
    pub ix_prefix: I,
    pub lst_calc_prog: T,
    pub lst_calc: C,
    pub pricing_prog: T,
    pub pricing: P,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxArgs<T, I, C, P> {
    pub lst_index: u32,

    /// In terms of
    /// - LST tokens for AddLiquidity
    /// - LP tokens for RemoveLiquidity
    pub amount: u64,

    /// In terms of
    /// - LP tokens for AddLiquidity
    /// - LST tokens for RemoveLiquidity
    pub min_out: u64,

    pub accs: IxAccs<T, I, C, P>,
}

impl<T, I, C: SolValCalcAccs, P> IxArgs<T, I, C, P> {
    #[inline]
    pub fn to_full(&self) -> inf1_ctl_core_liquidity::IxArgs {
        let Self {
            lst_index,
            amount,
            min_out,
            accs: IxAccs { lst_calc, .. },
        } = self;
        inf1_ctl_core_liquidity::IxArgs {
            // +1 for program account
            lst_value_calc_accs: lst_calc.suf_len() + 1,
            lst_index: *lst_index,
            amount: *amount,
            min_out: *min_out,
        }
    }
}

pub type AccsIter<'a, T> = Chain<
    Chain<Chain<Chain<slice::Iter<'a, T>, Once<&'a T>>, slice::Iter<'a, T>>, Once<&'a T>>,
    slice::Iter<'a, T>,
>;

impl<T, I: AsRef<[T]>, C: AsRef<[T]>, P: AsRef<[T]>> IxAccs<T, I, C, P> {
    #[inline]
    pub fn seq(&self) -> AccsIter<'_, T> {
        let Self {
            ix_prefix,
            lst_calc_prog,
            lst_calc,
            pricing_prog,
            pricing,
        } = self;
        ix_prefix
            .as_ref()
            .iter()
            .chain(once(lst_calc_prog))
            .chain(lst_calc.as_ref())
            .chain(once(pricing_prog))
            .chain(pricing.as_ref())
    }
}

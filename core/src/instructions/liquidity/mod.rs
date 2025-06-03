use core::{iter::Chain, slice};

use inf1_ctl_core::instructions::liquidity as inf1_ctl_core_liquidity;
use inf1_svc_core::traits::SolValCalcAccs;

pub mod add;
pub mod remove;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxAccs<I, C, P> {
    pub ix_prefix: I,
    pub lst_calc: C,
    pub pricing: P,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxArgs<C, P> {
    pub lst_index: u32,

    /// In terms of
    /// - LST tokens for AddLiquidity
    /// - LP tokens for RemoveLiquidity
    pub amount: u64,

    /// In terms of
    /// - LP tokens for AddLiquidity
    /// - LST tokens for RemoveLiquidity
    pub min_out: u64,

    pub accs: IxAccs<inf1_ctl_core_liquidity::IxPreKeysOwned, C, P>,
}

impl<C: SolValCalcAccs, P> IxArgs<C, P> {
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

pub type AccsIter<'a, T> = Chain<Chain<slice::Iter<'a, T>, slice::Iter<'a, T>>, slice::Iter<'a, T>>;

pub fn liquidity_ix_accs_seq<T, I: AsRef<[T]>, C: AsRef<[T]>, P: AsRef<[T]>>(
    IxAccs {
        ix_prefix,
        lst_calc,
        pricing,
    }: &IxAccs<I, C, P>,
) -> AccsIter<'_, T> {
    ix_prefix
        .as_ref()
        .iter()
        .chain(lst_calc.as_ref())
        .chain(pricing.as_ref())
}

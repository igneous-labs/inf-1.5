use core::{
    iter::{once, Chain, Once},
    slice,
};

use inf1_ctl_core::instructions::rebalance::start::{
    self as inf1_ctl_core_start_rebalance, StartRebalanceIxPreAccFlags,
    StartRebalanceIxPreKeysOwned, START_REBALANCE_IX_PRE_IS_SIGNER,
    START_REBALANCE_IX_PRE_IS_WRITER,
};
use inf1_svc_core::traits::SolValCalcAccs;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StartRebalanceIxAccs<T, I, C, D> {
    pub ix_prefix: I,
    pub out_calc_prog: T,
    pub out_calc: C,
    pub inp_calc_prog: T,
    pub inp_calc: D,
}

pub type AccsIter<'a, T> = Chain<
    Chain<Chain<Chain<slice::Iter<'a, T>, Once<&'a T>>, slice::Iter<'a, T>>, Once<&'a T>>,
    slice::Iter<'a, T>,
>;

impl<T, I: AsRef<[T]>, C: AsRef<[T]>, D: AsRef<[T]>> StartRebalanceIxAccs<T, I, C, D> {
    #[inline]
    pub fn seq(&self) -> AccsIter<'_, T> {
        let Self {
            ix_prefix,
            out_calc_prog,
            out_calc,
            inp_calc_prog,
            inp_calc,
        } = self;
        ix_prefix
            .as_ref()
            .iter()
            .chain(once(out_calc_prog))
            .chain(out_calc.as_ref())
            .chain(once(inp_calc_prog))
            .chain(inp_calc.as_ref())
    }
}

impl<C: SolValCalcAccs, D: SolValCalcAccs>
    StartRebalanceIxAccs<[u8; 32], StartRebalanceIxPreKeysOwned, C, D>
{
    /// Call [`Self::seq`] on return value to create iterator
    #[inline]
    pub fn keys_owned(
        &self,
    ) -> StartRebalanceIxAccs<[u8; 32], StartRebalanceIxPreKeysOwned, C::KeysOwned, D::KeysOwned>
    {
        let Self {
            ix_prefix,
            out_calc_prog,
            out_calc,
            inp_calc_prog,
            inp_calc,
        } = self;
        StartRebalanceIxAccs {
            ix_prefix: *ix_prefix,
            out_calc_prog: *out_calc_prog,
            out_calc: out_calc.suf_keys_owned(),
            inp_calc_prog: *inp_calc_prog,
            inp_calc: inp_calc.suf_keys_owned(),
        }
    }
}

impl<T, I, C: SolValCalcAccs, D: SolValCalcAccs> StartRebalanceIxAccs<T, I, C, D> {
    /// Call [`Self::seq`] on return value to create iterator
    #[inline]
    pub fn is_signer(
        &self,
    ) -> StartRebalanceIxAccs<bool, StartRebalanceIxPreAccFlags, C::AccFlags, D::AccFlags> {
        let Self {
            out_calc, inp_calc, ..
        } = self;
        StartRebalanceIxAccs {
            ix_prefix: START_REBALANCE_IX_PRE_IS_SIGNER,
            out_calc_prog: false,
            out_calc: out_calc.suf_is_signer(),
            inp_calc_prog: false,
            inp_calc: inp_calc.suf_is_signer(),
        }
    }

    /// Call [`Self::seq`] on return value to create iterator
    #[inline]
    pub fn is_writer(
        &self,
    ) -> StartRebalanceIxAccs<bool, StartRebalanceIxPreAccFlags, C::AccFlags, D::AccFlags> {
        let Self {
            out_calc, inp_calc, ..
        } = self;
        StartRebalanceIxAccs {
            ix_prefix: START_REBALANCE_IX_PRE_IS_WRITER,
            out_calc_prog: false,
            out_calc: out_calc.suf_is_writer(),
            inp_calc_prog: false,
            inp_calc: inp_calc.suf_is_writer(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StartRebalanceIxArgs<T, I, C, D> {
    pub out_lst_index: u32,
    pub inp_lst_index: u32,
    pub amount: u64,
    pub min_starting_out_lst: u64,
    pub max_starting_inp_lst: u64,
    pub accs: StartRebalanceIxAccs<T, I, C, D>,
}

impl<T, I, C: SolValCalcAccs, D: SolValCalcAccs> StartRebalanceIxArgs<T, I, C, D> {
    #[inline]
    pub fn to_full(&self) -> inf1_ctl_core_start_rebalance::StartRebalanceIxArgs {
        let Self {
            out_lst_index,
            inp_lst_index,
            amount,
            min_starting_out_lst,
            max_starting_inp_lst,
            accs: StartRebalanceIxAccs { out_calc, .. },
        } = self;
        inf1_ctl_core_start_rebalance::StartRebalanceIxArgs {
            // +1 for program account
            out_lst_value_calc_accs: out_calc.suf_len() + 1,
            out_lst_index: *out_lst_index,
            inp_lst_index: *inp_lst_index,
            amount: *amount,
            min_starting_out_lst: *min_starting_out_lst,
            max_starting_inp_lst: *max_starting_inp_lst,
        }
    }
}

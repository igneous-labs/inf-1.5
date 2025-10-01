use core::{
    iter::{once, Chain, Once},
    slice,
};

use inf1_ctl_core::instructions::rebalance::{
    end::{
        EndRebalanceIxPreAccFlags, EndRebalanceIxPreAccs, EndRebalanceIxPreKeysOwned,
        END_REBALANCE_IX_PRE_IS_SIGNER, END_REBALANCE_IX_PRE_IS_WRITER,
    },
    start::StartRebalanceIxPreAccs,
};
use inf1_svc_core::traits::SolValCalcAccs;

use crate::instructions::rebalance::start::StartRebalanceIxAccs;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndRebalanceIxAccs<T, I, C> {
    pub ix_prefix: I,
    pub inp_calc_prog: T,
    pub inp_calc: C,
}

impl<T: Copy, C> EndRebalanceIxAccs<T, EndRebalanceIxPreAccs<T>, C> {
    #[inline]
    pub fn from_start<X>(
        StartRebalanceIxAccs {
            ix_prefix,
            inp_calc_prog,
            inp_calc,
            ..
        }: StartRebalanceIxAccs<T, StartRebalanceIxPreAccs<T>, X, C>,
    ) -> Self {
        Self {
            ix_prefix: EndRebalanceIxPreAccs::from_start(ix_prefix),
            inp_calc_prog,
            inp_calc,
        }
    }
}

pub type AccsIter<'a, T> = Chain<Chain<slice::Iter<'a, T>, Once<&'a T>>, slice::Iter<'a, T>>;

impl<T, I: AsRef<[T]>, C: AsRef<[T]>> EndRebalanceIxAccs<T, I, C> {
    #[inline]
    pub fn seq(&self) -> AccsIter<'_, T> {
        let Self {
            ix_prefix,
            inp_calc_prog,
            inp_calc,
        } = self;
        ix_prefix
            .as_ref()
            .iter()
            .chain(once(inp_calc_prog))
            .chain(inp_calc.as_ref())
    }
}

impl<C: SolValCalcAccs> EndRebalanceIxAccs<[u8; 32], EndRebalanceIxPreKeysOwned, C> {
    /// Call [`Self::seq`] on return value to create iterator
    #[inline]
    pub fn keys_owned(
        &self,
    ) -> EndRebalanceIxAccs<[u8; 32], EndRebalanceIxPreKeysOwned, C::KeysOwned> {
        let Self {
            ix_prefix,
            inp_calc_prog,
            inp_calc,
        } = self;
        EndRebalanceIxAccs {
            ix_prefix: *ix_prefix,
            inp_calc_prog: *inp_calc_prog,
            inp_calc: inp_calc.suf_keys_owned(),
        }
    }
}

impl<T, I, C: SolValCalcAccs> EndRebalanceIxAccs<T, I, C> {
    /// Call [`Self::seq`] on return value to create iterator
    #[inline]
    pub fn is_signer(&self) -> EndRebalanceIxAccs<bool, EndRebalanceIxPreAccFlags, C::AccFlags> {
        let Self { inp_calc, .. } = self;
        EndRebalanceIxAccs {
            ix_prefix: END_REBALANCE_IX_PRE_IS_SIGNER,
            inp_calc_prog: false,
            inp_calc: inp_calc.suf_is_signer(),
        }
    }

    /// Call [`Self::seq`] on return value to create iterator
    #[inline]
    pub fn is_writer(&self) -> EndRebalanceIxAccs<bool, EndRebalanceIxPreAccFlags, C::AccFlags> {
        let Self { inp_calc, .. } = self;
        EndRebalanceIxAccs {
            ix_prefix: END_REBALANCE_IX_PRE_IS_WRITER,
            inp_calc_prog: false,
            inp_calc: inp_calc.suf_is_writer(),
        }
    }
}

use core::{
    iter::{once, Chain, Once},
    slice,
};

use inf1_ctl_core::instructions::sync_sol_value::{
    SyncSolValueIxPreAccFlags, SyncSolValueIxPreKeysOwned, SYNC_SOL_VALUE_IX_PRE_IS_SIGNER,
    SYNC_SOL_VALUE_IX_PRE_IS_WRITER,
};
use inf1_svc_core::traits::SolValCalcAccs;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyncSolValueIxAccs<T, I, C> {
    pub ix_prefix: I,
    pub calc_prog: T,
    pub calc: C,
}

pub type SyncSolValueAccsIter<'a, T> =
    Chain<Chain<slice::Iter<'a, T>, Once<&'a T>>, slice::Iter<'a, T>>;

impl<T, I: AsRef<[T]>, C: AsRef<[T]>> SyncSolValueIxAccs<T, I, C> {
    #[inline]
    pub fn seq(&self) -> SyncSolValueAccsIter<'_, T> {
        let Self {
            ix_prefix,
            calc_prog,
            calc,
        } = self;
        ix_prefix
            .as_ref()
            .iter()
            .chain(once(calc_prog))
            .chain(calc.as_ref())
    }
}

/// Call [`SyncSolValueIxAccs::seq`] on return value to create iterator
#[inline]
pub fn sync_sol_value_ix_keys_owned<C: SolValCalcAccs>(
    SyncSolValueIxAccs {
        ix_prefix,
        calc_prog,
        calc,
    }: &SyncSolValueIxAccs<[u8; 32], SyncSolValueIxPreKeysOwned, C>,
) -> SyncSolValueIxAccs<[u8; 32], SyncSolValueIxPreKeysOwned, C::KeysOwned> {
    SyncSolValueIxAccs {
        ix_prefix: *ix_prefix,
        calc_prog: *calc_prog,
        calc: calc.suf_keys_owned(),
    }
}

/// Call [`SyncSolValueIxAccs::seq`] on return value to create iterator
#[inline]
pub fn sync_sol_value_ix_is_signer<T, I, C: SolValCalcAccs>(
    SyncSolValueIxAccs { calc, .. }: &SyncSolValueIxAccs<T, I, C>,
) -> SyncSolValueIxAccs<bool, SyncSolValueIxPreAccFlags, C::AccFlags> {
    SyncSolValueIxAccs {
        ix_prefix: SYNC_SOL_VALUE_IX_PRE_IS_SIGNER,
        calc_prog: false,
        calc: calc.suf_is_signer(),
    }
}

/// Call [`SyncSolValueIxAccs::seq`] on return value to create iterator
#[inline]
pub fn sync_sol_value_ix_is_writer<T, I, C: SolValCalcAccs>(
    SyncSolValueIxAccs { calc, .. }: &SyncSolValueIxAccs<T, I, C>,
) -> SyncSolValueIxAccs<bool, SyncSolValueIxPreAccFlags, C::AccFlags> {
    SyncSolValueIxAccs {
        ix_prefix: SYNC_SOL_VALUE_IX_PRE_IS_WRITER,
        calc_prog: false,
        calc: calc.suf_is_writer(),
    }
}

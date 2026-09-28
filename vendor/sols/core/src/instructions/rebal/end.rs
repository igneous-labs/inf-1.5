use generic_array_struct::generic_array_struct;

use crate::{
    const_map_rebal_sol_empty,
    instructions::{
        common::PoolHoldingAccFlags,
        internal_utils::{csi_at, DiscmOnlyIxData},
        rebal::{RebalPoolHolding, RebalSvcProgAcc, StartRebalIxAccs, StartRebalIxPreAccs},
        user::ClaimHoldingIxSufAccs,
    },
    internal_utils::impl_memset,
    typedefs::TokenSolEmpty,
    utils::InpOut,
};

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndRebalIxPreAccs<T> {
    /// Pool being rebalanced
    pub pool: T,

    /// Portfolio of `pool`
    pub portfolio: T,
}

impl_memset!(EndRebalIxPreAccs);

pub type EndRebalIxPreKeys<'a> = EndRebalIxPreAccs<&'a [u8; 32]>;
pub type EndRebalIxPreKeysOwned = EndRebalIxPreAccs<[u8; 32]>;
pub type EndRebalIxPreAccFlags = EndRebalIxPreAccs<bool>;

pub const END_REBAL_IX_PRE_IS_WRITER: EndRebalIxPreAccFlags = EndRebalIxPreAccFlags::memset(true);

pub const END_REBAL_IX_PRE_IS_SIGNER: EndRebalIxPreAccFlags = EndRebalIxPreAccFlags::memset(false);

pub const END_REBAL_IX_HOLDING_IS_WRITER: PoolHoldingAccFlags = PoolHoldingAccFlags::memset(false);

pub const END_REBAL_IX_HOLDING_IS_SIGNER: PoolHoldingAccFlags = PoolHoldingAccFlags::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EndRebalIxAccs<P, H, S, X> {
    /// [`EndRebalIxPreAccs`]
    pub pre: P,

    /// [`RebalPoolHolding`]
    pub inp: H,

    /// input holding's SOL value calculator account suffix,
    /// excluding mint. Omitted if SOL
    pub inp_svc_suf: S,

    /// [`RebalSvcProgAcc`]
    pub inp_svc_prog: X,
}

pub type EndRebalIxGen<T, S> =
    EndRebalIxAccs<EndRebalIxPreAccs<T>, RebalPoolHolding<T>, S, RebalSvcProgAcc<T>>;

#[inline]
pub const fn end_rebal_ix_is_writer<S>(
    inp_svc_suf: TokenSolEmpty<S>,
) -> EndRebalIxGen<bool, TokenSolEmpty<S>> {
    let inp_svc_prog = const_map_rebal_sol_empty!(inp_svc_suf, [false]);
    let inp = const_map_rebal_sol_empty!(inp_svc_suf, END_REBAL_IX_HOLDING_IS_WRITER);
    EndRebalIxAccs {
        pre: END_REBAL_IX_PRE_IS_WRITER,
        inp,
        inp_svc_suf,
        inp_svc_prog,
    }
}

#[inline]
pub const fn end_rebal_ix_is_signer<S>(
    inp_svc_suf: TokenSolEmpty<S>,
) -> EndRebalIxGen<bool, TokenSolEmpty<S>> {
    let inp_svc_prog = const_map_rebal_sol_empty!(inp_svc_suf, [false]);
    let inp = const_map_rebal_sol_empty!(inp_svc_suf, END_REBAL_IX_HOLDING_IS_SIGNER);
    EndRebalIxAccs {
        pre: END_REBAL_IX_PRE_IS_SIGNER,
        inp,
        inp_svc_suf,
        inp_svc_prog,
    }
}

impl<P, H, S, X> EndRebalIxAccs<P, H, S, X> {
    #[inline]
    pub const fn is_writer(inp_svc_suf: TokenSolEmpty<S>) -> EndRebalIxGen<bool, TokenSolEmpty<S>> {
        end_rebal_ix_is_writer(inp_svc_suf)
    }

    #[inline]
    pub const fn is_signer(inp_svc_suf: TokenSolEmpty<S>) -> EndRebalIxGen<bool, TokenSolEmpty<S>> {
        end_rebal_ix_is_signer(inp_svc_suf)
    }
}

pub type FromStartRebalIxAccs<P, H, S, O, X, Y> = StartRebalIxAccs<
    StartRebalIxPreAccs<P>,
    ClaimHoldingIxSufAccs<P>,
    InpOut<RebalPoolHolding<H>>,
    S,
    O,
    X,
    Y,
>;

#[inline]
pub const fn end_rebal_ix_accs_from_start_copied<
    P: Copy,
    H: Copy,
    S: Copy,
    O: Copy,
    X: Copy,
    Y: Copy,
>(
    StartRebalIxAccs {
        pre,
        suf,
        holdings,
        inp_svc_suf,
        inp_svc_prog,
        out_svc_suf: _,
        out_progs: _,
    }: FromStartRebalIxAccs<P, H, S, O, X, Y>,
) -> EndRebalIxAccs<EndRebalIxPreAccs<P>, RebalPoolHolding<H>, S, X> {
    EndRebalIxAccs {
        pre: EndRebalIxPreAccs::const_from_destr(EndRebalIxPreAccsDestr {
            pool: *pre.pool(),
            portfolio: *suf.portfolio(),
        }),
        inp: *holdings.inp(),
        inp_svc_suf,
        inp_svc_prog,
    }
}

impl<P: Copy, H: Copy, S: Copy, X: Copy>
    EndRebalIxAccs<EndRebalIxPreAccs<P>, RebalPoolHolding<H>, S, X>
{
    #[inline]
    pub const fn from_start_copied<O: Copy, Y: Copy>(
        start: FromStartRebalIxAccs<P, H, S, O, X, Y>,
    ) -> Self {
        end_rebal_ix_accs_from_start_copied(start)
    }
}

#[inline]
pub fn end_rebal_ix_accs_from_start_cloned<P: Clone, H: Clone, S: Clone, O, X: Clone, Y>(
    StartRebalIxAccs {
        pre,
        suf,
        holdings,
        inp_svc_suf,
        inp_svc_prog,
        out_svc_suf: _,
        out_progs: _,
    }: &FromStartRebalIxAccs<P, H, S, O, X, Y>,
) -> EndRebalIxAccs<EndRebalIxPreAccs<P>, RebalPoolHolding<H>, S, X> {
    EndRebalIxAccs {
        pre: EndRebalIxPreAccs::from_destr(EndRebalIxPreAccsDestr {
            pool: pre.pool().clone(),
            portfolio: suf.portfolio().clone(),
        }),
        inp: holdings.inp().clone(),
        inp_svc_suf: inp_svc_suf.clone(),
        inp_svc_prog: inp_svc_prog.clone(),
    }
}

impl<P: Clone, H: Clone, S: Clone, X: Clone>
    EndRebalIxAccs<EndRebalIxPreAccs<P>, RebalPoolHolding<H>, S, X>
{
    #[inline]
    pub fn from_start_cloned<O, Y>(start: &FromStartRebalIxAccs<P, H, S, O, X, Y>) -> Self {
        end_rebal_ix_accs_from_start_cloned(start)
    }
}

pub type EndRebalIxAccsIter<'a, T> = csi_at!(@ @ @);

pub type EndRebalIxSvcAccsIter<'a, T> = csi_at!(@);

impl<T, S> EndRebalIxGen<T, S>
where
    S: AsRef<[T]>,
{
    #[inline]
    pub fn seq(&self) -> EndRebalIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.inp.as_ref().iter())
            .chain(self.inp_svc_suf.as_ref().iter())
            .chain(self.inp_svc_prog.as_ref().iter())
    }

    /// Returns the sequence of accounts to input to the
    /// SOL value calculator program CPI for the input token
    #[inline]
    pub fn inp_svc_accs(&self) -> EndRebalIxSvcAccsIter<'_, T> {
        match &self.inp {
            TokenSolEmpty::Sol(_) => [].iter().chain([].iter()),
            TokenSolEmpty::Other(inp) => core::slice::from_ref(inp.mint())
                .iter()
                .chain(self.inp_svc_suf.as_ref().iter()),
        }
    }
}

// Data

pub const END_REBAL_IX_DISCM: u8 = 18;

pub const END_REBAL_IX_DATA_LEN: usize = EndRebalIxData::LEN;

pub type EndRebalIxData = DiscmOnlyIxData<END_REBAL_IX_DISCM>;

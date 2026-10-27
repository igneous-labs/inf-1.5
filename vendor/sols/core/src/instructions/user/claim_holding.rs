use crate::{
    instructions::{
        common::{
            HoldingProgAccs, PoolHolding, PoolHoldingAccFlags, HOLDING_PROG_IS_SIGNER,
            HOLDING_PROG_IS_WRITER,
        },
        internal_utils::{csi_at, SwapIxData},
        user::{MintClaimCoreAccs, CLAIM_IX_IS_SIGNER, CLAIM_IX_IS_WRITER},
    },
    internal_utils::impl_memset,
};

use generic_array_struct::generic_array_struct;

// Accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ClaimHoldingIxSufAccs<T> {
    /// Pool's portfolio acc
    pub portfolio: T,

    /// Protocol PDA
    pub protocol: T,
}

impl_memset!(ClaimHoldingIxSufAccs);

pub type ClaimHoldingIxSufKeys<'a> = ClaimHoldingIxSufAccs<&'a [u8; 32]>;
pub type ClaimHoldingIxSufKeysOwned = ClaimHoldingIxSufAccs<[u8; 32]>;
pub type ClaimHoldingIxSufAccFlags = ClaimHoldingIxSufAccs<bool>;

pub const CLAIM_HOLDING_IX_SUF_IS_WRITER: ClaimHoldingIxSufAccFlags =
    ClaimHoldingIxSufAccFlags::memset(false).const_with_portfolio(true);

pub const CLAIM_HOLDING_IX_SUF_IS_SIGNER: ClaimHoldingIxSufAccFlags =
    ClaimHoldingIxSufAccFlags::memset(false);

pub const CLAIM_HOLDING_IX_HOLDING_IS_WRITER: PoolHoldingAccFlags =
    PoolHoldingAccFlags::memset(false).const_with_ata(true);

pub const CLAIM_HOLDING_IX_HOLDING_IS_SIGNER: PoolHoldingAccFlags =
    PoolHoldingAccFlags::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClaimHoldingIxAccs<P, S, H, V, X> {
    /// [`MintClaimCoreAccs`]
    pub pre: P,

    /// [`ClaimHoldingIxSufAccs`]
    pub suf: S,

    /// [`PoolHolding`]
    pub holding: H,

    /// holding's SOL value calculator account suffix,
    /// excluding mint
    pub svc_suf: V,

    /// [`HoldingProgAccs`]
    pub progs: X,
}

pub type ClaimHoldingIxGen<T, V> = ClaimHoldingIxAccs<
    MintClaimCoreAccs<T>,
    ClaimHoldingIxSufAccs<T>,
    PoolHolding<T>,
    V,
    HoldingProgAccs<T>,
>;

#[inline]
pub const fn claim_holding_ix_is_writer<V>(svc_suf: V) -> ClaimHoldingIxGen<bool, V> {
    ClaimHoldingIxAccs {
        pre: CLAIM_IX_IS_WRITER,
        suf: CLAIM_HOLDING_IX_SUF_IS_WRITER,
        holding: CLAIM_HOLDING_IX_HOLDING_IS_WRITER,
        svc_suf,
        progs: HOLDING_PROG_IS_WRITER,
    }
}

#[inline]
pub const fn claim_holding_ix_is_signer<V>(svc_suf: V) -> ClaimHoldingIxGen<bool, V> {
    ClaimHoldingIxAccs {
        pre: CLAIM_IX_IS_SIGNER,
        suf: CLAIM_HOLDING_IX_SUF_IS_SIGNER,
        holding: CLAIM_HOLDING_IX_HOLDING_IS_SIGNER,
        svc_suf,
        progs: HOLDING_PROG_IS_SIGNER,
    }
}

impl<P, S, H, V, X> ClaimHoldingIxAccs<P, S, H, V, X> {
    #[inline]
    pub const fn is_writer(svc_suf: V) -> ClaimHoldingIxGen<bool, V> {
        claim_holding_ix_is_writer(svc_suf)
    }

    #[inline]
    pub const fn is_signer(svc_suf: V) -> ClaimHoldingIxGen<bool, V> {
        claim_holding_ix_is_signer(svc_suf)
    }
}

pub type ClaimHoldingIxAccsIter<'a, T> = csi_at!(@ @ @ @);

pub type ClaimHoldingIxSvcAccsIter<'a, T> = csi_at!(@);

impl<T, V: AsRef<[T]>> ClaimHoldingIxGen<T, V> {
    #[inline]
    pub fn seq(&self) -> ClaimHoldingIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.suf.0.iter())
            .chain(self.holding.0.iter())
            .chain(self.svc_suf.as_ref().iter())
            .chain(self.progs.0.iter())
    }

    /// Returns the sequence of accounts to input to the
    /// SOL value calculator program CPI
    #[inline]
    pub fn svc_accs(&self) -> ClaimHoldingIxSvcAccsIter<'_, T> {
        core::slice::from_ref(self.holding.mint())
            .iter()
            .chain(self.svc_suf.as_ref().iter())
    }
}

// Data

pub const CLAIM_HOLDING_IX_DISCM: u8 = 8;

pub const CLAIM_HOLDING_IX_DATA_LEN: usize = ClaimHoldingIxData::LEN;

pub type ClaimHoldingIxData = SwapIxData<CLAIM_HOLDING_IX_DISCM>;

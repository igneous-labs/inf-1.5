use generic_array_struct::generic_array_struct;

use crate::{
    instructions::{
        common::{PoolHolding, SvcProgWhitelistAccs},
        internal_utils::{csi_at, NanoIxData},
        manager::{
            CrudHoldingIxPreAccs, CRUD_HOLDING_IX_PRE_IS_SIGNER, CRUD_HOLDING_IX_PRE_IS_WRITER,
        },
    },
    internal_utils::{impl_asref, impl_memset},
};

// Accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetHoldingIxProgAccs<T> {
    /// holding's token program
    pub token: T,

    /// system program
    pub system: T,

    /// associated token program
    pub ata: T,
}

pub type SetHoldingIxProgKeys<'a> = SetHoldingIxProgAccs<&'a [u8; 32]>;
pub type SetHoldingIxProgKeysOwned = SetHoldingIxProgAccs<[u8; 32]>;
pub type SetHoldingIxProgAccFlags = SetHoldingIxProgAccs<bool>;

impl_memset!(SetHoldingIxProgAccs);
impl_asref!(SetHoldingIxProgAccs);

pub const SET_HOLDING_IX_PROG_IS_WRITER: SetHoldingIxProgAccFlags =
    SetHoldingIxProgAccFlags::memset(false);

pub const SET_HOLDING_IX_PROG_IS_SIGNER: SetHoldingIxProgAccFlags =
    SetHoldingIxProgAccFlags::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetHoldingIxAccs<P, H, S, X, U> {
    /// [`CrudHoldingIxPreAccs`]
    pub pre: P,

    /// [`PoolHolding`]
    pub holding: H,

    /// [`SvcProgWhitelistAccs`]
    pub svc: S,

    /// [`SetHoldingIxProgAccs`]
    ///
    /// Need to include these program accounts instead of as a final suffix
    /// like the rest of the instructions because:
    /// - makes handling of variable length svc_suf easier
    /// - system and token account need to be passed in to createIdempotent ATA CPI
    pub progs: X,

    /// holding's SOL value calculator account suffix,
    /// excluding mint
    pub svc_suf: U,
}

pub type SetHoldingIxGen<T, U> = SetHoldingIxAccs<
    CrudHoldingIxPreAccs<T>,
    PoolHolding<T>,
    SvcProgWhitelistAccs<T>,
    SetHoldingIxProgAccs<T>,
    U,
>;

#[inline]
pub const fn set_holding_ix_is_writer<U>(svc_suf: U) -> SetHoldingIxGen<bool, U> {
    SetHoldingIxGen {
        pre: CRUD_HOLDING_IX_PRE_IS_WRITER,
        holding: PoolHolding::memset(false).const_with_ata(true),
        svc: SvcProgWhitelistAccs::memset(false),
        progs: SET_HOLDING_IX_PROG_IS_WRITER,
        svc_suf,
    }
}

#[inline]
pub const fn set_holding_ix_is_signer<U>(svc_suf: U) -> SetHoldingIxGen<bool, U> {
    SetHoldingIxGen {
        pre: CRUD_HOLDING_IX_PRE_IS_SIGNER,
        holding: PoolHolding::memset(false),
        svc: SvcProgWhitelistAccs::memset(false),
        progs: SET_HOLDING_IX_PROG_IS_SIGNER,
        svc_suf,
    }
}

impl<P, H, S, X, U> SetHoldingIxAccs<P, H, S, X, U> {
    #[inline]
    pub const fn is_writer(svc_suf: U) -> SetHoldingIxGen<bool, U> {
        set_holding_ix_is_writer(svc_suf)
    }

    #[inline]
    pub const fn is_signer(svc_suf: U) -> SetHoldingIxGen<bool, U> {
        set_holding_ix_is_signer(svc_suf)
    }
}

pub type SetHoldingIxAccsIter<'a, T> = csi_at!(@ @ @ @);

impl<T, U> SetHoldingIxGen<T, U>
where
    U: AsRef<[T]>,
{
    #[inline]
    pub fn seq(&self) -> SetHoldingIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.holding.0.iter())
            .chain(self.svc.0.iter())
            .chain(self.progs.0.iter())
            .chain(self.svc_suf.as_ref().iter())
    }
}

// Data

pub const SET_HOLDING_IX_DISCM: u8 = 13;

pub const SET_HOLDING_IX_DATA_LEN: usize = SetHoldingIxData::LEN;

pub type SetHoldingIxData = NanoIxData<SET_HOLDING_IX_DISCM>;

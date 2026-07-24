use super::common::AdminIxPreAccs;
use crate::instructions::{csi_at, generic::DiscmOnlyIxData};

// Accounts

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetAdminIxAccs<P, T> {
    /// [`AdminIxPreAccs`]
    pub pre: P,

    /// New admin address to store
    pub new_admin: T,
}

pub type SetAdminIxAccsGen<T> = SetAdminIxAccs<AdminIxPreAccs<T>, T>;

pub const SET_ADMIN_IX_IS_WRITER: SetAdminIxAccsGen<bool> = SetAdminIxAccs {
    pre: super::common::ADMIN_IX_PRE_IS_WRITER,
    new_admin: false,
};

pub const SET_ADMIN_IX_IS_SIGNER: SetAdminIxAccsGen<bool> = SetAdminIxAccs {
    pre: super::common::ADMIN_IX_PRE_IS_SIGNER,
    new_admin: false,
};

pub type SetAdminAccsIter<'a, T> = csi_at!(@);

impl<T> SetAdminIxAccsGen<T> {
    #[inline]
    pub fn seq(&self) -> SetAdminAccsIter<'_, T> {
        let Self { pre, new_admin } = self;
        pre.0.iter().chain(core::slice::from_ref(new_admin).iter())
    }
}

// Data

pub const SET_ADMIN_IX_DISCM: u8 = 254;

pub type SetAdminIxData = DiscmOnlyIxData<SET_ADMIN_IX_DISCM>;

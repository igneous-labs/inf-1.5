use generic_array_struct::generic_array_struct;

use crate::internal_utils::{impl_asref, impl_memset};

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CrudSvcWhitelistAccs<T> {
    /// Protocol PDA
    pub protocol: T,

    /// SVC whitelist auth
    pub auth: T,

    /// The SVC program being added or removed
    pub prog: T,
}

impl_memset!(CrudSvcWhitelistAccs);
impl_asref!(CrudSvcWhitelistAccs);

pub type CrudSvcWhitelistKeys<'a> = CrudSvcWhitelistAccs<&'a [u8; 32]>;
pub type CrudSvcWhitelistKeysOwned = CrudSvcWhitelistAccs<[u8; 32]>;
pub type CrudSvcWhitelistAccFlags = CrudSvcWhitelistAccs<bool>;

pub const CRUD_SVC_WHITELIST_IS_WRITER: CrudSvcWhitelistAccFlags =
    CrudSvcWhitelistAccs::memset(false).const_with_protocol(true);

pub const CRUD_SVC_WHITELIST_IS_SIGNER: CrudSvcWhitelistAccFlags =
    CrudSvcWhitelistAccs::memset(false).const_with_auth(true);

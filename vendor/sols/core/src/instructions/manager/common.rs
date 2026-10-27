use generic_array_struct::generic_array_struct;

use crate::internal_utils::{impl_asref, impl_memset};

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CrudHoldingIxPreAccs<T> {
    /// Pool's portfolio acc
    pub pool: T,

    /// [`Self::pool`]'s portfolio
    pub portfolio: T,

    /// [`Self::pool`]'s manager
    pub manager: T,
}

pub type CrudHoldingIxPreKeys<'a> = CrudHoldingIxPreAccs<&'a [u8; 32]>;
pub type CrudHoldingIxPreKeysOwned = CrudHoldingIxPreAccs<[u8; 32]>;
pub type CrudHoldingIxPreAccFlags = CrudHoldingIxPreAccs<bool>;

impl_memset!(CrudHoldingIxPreAccs);
impl_asref!(CrudHoldingIxPreAccs);

pub const CRUD_HOLDING_IX_PRE_IS_WRITER: CrudHoldingIxPreAccFlags =
    CrudHoldingIxPreAccFlags::memset(false).const_with_portfolio(true);

pub const CRUD_HOLDING_IX_PRE_IS_SIGNER: CrudHoldingIxPreAccFlags =
    CrudHoldingIxPreAccFlags::memset(false).const_with_manager(true);

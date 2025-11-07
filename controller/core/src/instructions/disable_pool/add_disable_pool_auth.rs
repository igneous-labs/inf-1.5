use generic_array_struct::generic_array_struct;

use crate::instructions::generic::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AddDisablePoolAuthIxAccs<T> {
    /// Account paying for additional rent
    pub payer: T,

    /// Pool's admin
    pub admin: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// New disable pool authority to add
    pub new: T,

    /// The DisablePoolAuthority list singleton PDA
    pub disable_pool_auth_list: T,

    /// System program
    pub system_program: T,
}

impl<T: Copy> AddDisablePoolAuthIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; ADD_DISABLE_POOL_AUTH_IX_ACCS_LEN])
    }
}

pub type AddDisablePoolAuthIxKeys<'a> = AddDisablePoolAuthIxAccs<&'a [u8; 32]>;

pub type AddDisablePoolAuthIxKeysOwned = AddDisablePoolAuthIxAccs<[u8; 32]>;

pub type AddDisablePoolAuthIxAccFlags = AddDisablePoolAuthIxAccs<bool>;

pub const ADD_DISABLE_POOL_AUTH_IX_IS_WRITER: AddDisablePoolAuthIxAccFlags =
    AddDisablePoolAuthIxAccFlags::memset(false)
        .const_with_payer(true)
        .const_with_disable_pool_auth_list(true);

pub const ADD_DISABLE_POOL_AUTH_IX_IS_SIGNER: AddDisablePoolAuthIxAccFlags =
    AddDisablePoolAuthIxAccFlags::memset(false)
        .const_with_payer(true)
        .const_with_admin(true);

// Data

pub const ADD_DISABLE_POOL_AUTH_IX_DISCM: u8 = 15;

pub type AddDisablePoolAuthIxData = DiscmOnlyIxData<ADD_DISABLE_POOL_AUTH_IX_DISCM>;

pub const ADD_DISABLE_POOL_AUTH_IX_DATA_LEN: usize = AddDisablePoolAuthIxData::DATA_LEN;

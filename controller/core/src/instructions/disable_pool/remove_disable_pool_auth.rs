use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::{U32IxData, U32_IX_DATA_LEN};

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RemoveDisablePoolAuthIxAccs<T> {
    /// Account receiving lamports in excess of rent-exemption of
    /// DisablePoolAuthorityList after shrinkage
    pub refund_rent_to: T,

    /// Either pool's admin or `remove` itself
    pub signer: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// The disable pool authority to remove.
    ///
    /// This is here to ensure that the index argument matches up.
    pub remove: T,

    /// The DisablePoolAuthority list singleton PDA
    pub disable_pool_auth_list: T,
}

impl<T: Copy> RemoveDisablePoolAuthIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; REMOVE_DISABLE_POOL_AUTH_IX_ACCS_LEN])
    }
}

pub type RemoveDisablePoolAuthIxKeys<'a> = RemoveDisablePoolAuthIxAccs<&'a [u8; 32]>;

pub type RemoveDisablePoolAuthIxKeysOwned = RemoveDisablePoolAuthIxAccs<[u8; 32]>;

pub type RemoveDisablePoolAuthIxAccFlags = RemoveDisablePoolAuthIxAccs<bool>;

pub const REMOVE_DISABLE_POOL_AUTH_IX_IS_WRITER: RemoveDisablePoolAuthIxAccFlags =
    RemoveDisablePoolAuthIxAccFlags::memset(false)
        .const_with_refund_rent_to(true)
        .const_with_disable_pool_auth_list(true);

pub const REMOVE_DISABLE_POOL_AUTH_IX_IS_SIGNER: RemoveDisablePoolAuthIxAccFlags =
    RemoveDisablePoolAuthIxAccFlags::memset(false).const_with_signer(true);

// Data

pub const REMOVE_DISABLE_POOL_AUTH_IX_DISCM: u8 = 16;

pub const REMOVE_DISABLE_POOL_AUTH_IX_DATA_LEN: usize = U32_IX_DATA_LEN;

pub type RemoveDisablePoolAuthIxData = U32IxData<REMOVE_DISABLE_POOL_AUTH_IX_DISCM>;

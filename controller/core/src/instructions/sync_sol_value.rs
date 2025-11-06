use generic_array_struct::generic_array_struct;

use crate::instructions::generic::U32IxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SyncSolValueIxPreAccs<T> {
    /// Mint of the LST to sync SOL value for
    pub lst_mint: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// Dynamic list PDA of LstStates for each LST in the pool
    pub lst_state_list: T,

    /// LST reserves token account of the pool.
    ///
    /// The LST's SOL value calculator program suffix accounts follow.
    pub pool_reserves: T,
}

impl<T: Copy> SyncSolValueIxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SYNC_SOL_VALUE_IX_PRE_ACCS_LEN])
    }
}

pub type SyncSolValueIxPreKeys<'a> = SyncSolValueIxPreAccs<&'a [u8; 32]>;

pub type SyncSolValueIxPreKeysOwned = SyncSolValueIxPreAccs<[u8; 32]>;

pub type SyncSolValueIxPreAccFlags = SyncSolValueIxPreAccs<bool>;

impl<T> AsRef<[T]> for SyncSolValueIxPreAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const SYNC_SOL_VALUE_IX_PRE_IS_WRITER: SyncSolValueIxPreAccFlags =
    SyncSolValueIxPreAccFlags::memset(false)
        .const_with_pool_state(true)
        .const_with_lst_state_list(true);

pub const SYNC_SOL_VALUE_IX_PRE_IS_SIGNER: SyncSolValueIxPreAccFlags =
    SyncSolValueIxPreAccFlags::memset(false);

// Data

pub const SYNC_SOL_VALUE_IX_DISCM: u8 = 0;

pub const SYNC_SOL_VALUE_IX_DATA_LEN: usize = SyncSolValueIxData::DATA_LEN;

pub type SyncSolValueIxData = U32IxData<SYNC_SOL_VALUE_IX_DISCM>;

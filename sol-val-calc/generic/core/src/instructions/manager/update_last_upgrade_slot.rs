use generic_array_struct::generic_array_struct;

use crate::instructions::discm_only::DiscmOnlyIxData;

// Accounts

/// UpdateLastUpgradeSlot
#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ULUSIxAccs<T> {
    /// The program's manager
    pub manager: T,

    /// The program's State singleton PDA
    pub state: T,

    /// The stake pool program that this SVC program works with
    pub pool_prog: T,

    /// `pool_prog`'s BPF loader V3 program data PDA account
    pub pool_prog_data: T,
}

impl<T: Copy> ULUSIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; ULUS_IX_ACCS_LEN])
    }
}

pub type ULUSIxKeys<'a> = ULUSIxAccs<&'a [u8; 32]>;

pub type ULUSIxKeysOwned = ULUSIxAccs<[u8; 32]>;

pub type ULUSIxAccFlags = ULUSIxAccs<bool>;

impl<T> AsRef<[T]> for ULUSIxAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const ULUS_IX_IS_WRITER: ULUSIxAccFlags = ULUSIxAccFlags::memset(false).const_with_state(true);

pub const ULUS_IX_IS_SIGNER: ULUSIxAccFlags =
    ULUSIxAccFlags::memset(false).const_with_manager(true);

// Data

pub const ULUS_IX_DISCM: u8 = 253;

pub const ULUS_IX_DATA_LEN: usize = ULUSIxData::DATA_LEN;

pub type ULUSIxData = DiscmOnlyIxData<ULUS_IX_DISCM>;

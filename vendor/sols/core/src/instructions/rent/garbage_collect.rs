use generic_array_struct::generic_array_struct;

use crate::{
    instructions::internal_utils::DiscmOnlyIxData,
    internal_utils::{impl_asref, impl_memset},
};

// Accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GarbageCollectIxAccs<T> {
    /// Program account being garbage collected
    pub acc: T,

    /// Refund excess lamports to this account
    pub out: T,
}

pub type GarbageCollectIxKeys<'a> = GarbageCollectIxAccs<&'a [u8; 32]>;
pub type GarbageCollectIxOwned = GarbageCollectIxAccs<[u8; 32]>;
pub type GarbageCollectIxAccFlags = GarbageCollectIxAccs<bool>;

impl_memset!(GarbageCollectIxAccs);
impl_asref!(GarbageCollectIxAccs);

pub const GARBAGE_COLLECT_IX_IS_WRITER: GarbageCollectIxAccFlags =
    GarbageCollectIxAccFlags::memset(true);

pub const GARBAGE_COLLECT_IX_IS_SIGNER: GarbageCollectIxAccFlags =
    GarbageCollectIxAccFlags::memset(false);

// Data

pub const GARBAGE_COLLECT_IX_DISCM: u8 = 29;

pub const GARBAGE_COLLECT_IX_DATA_LEN: usize = GarbageCollectIxData::LEN;

pub type GarbageCollectIxData = DiscmOnlyIxData<GARBAGE_COLLECT_IX_DISCM>;

use crate::instructions::{common::MintPoolPairAccFlags, internal_utils::DiscmOnlyIxData};

// Accounts: just MintPoolPair

pub const SYNC_RENT_POOL_IX_IS_WRITER: MintPoolPairAccFlags =
    MintPoolPairAccFlags::memset(false).const_with_pool(true);

pub const SYNC_RENT_POOL_IX_IS_SIGNER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(false);

// Data

pub const SYNC_RENT_POOL_IX_DISCM: u8 = 28;

pub const SYNC_RENT_POOL_IX_DATA_LEN: usize = SyncRentPoolIxData::LEN;

pub type SyncRentPoolIxData = DiscmOnlyIxData<SYNC_RENT_POOL_IX_DISCM>;

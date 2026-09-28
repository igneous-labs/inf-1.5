use crate::instructions::{internal_utils::U64OptIxData, user::MintClaimCoreAccs};

// Accounts - just MintClaimCoreAccs

pub const CLAIM_IX_IS_WRITER: MintClaimCoreAccs<bool> =
    MintClaimCoreAccs::memset(true).const_with_mint(false);

pub const CLAIM_IX_IS_SIGNER: MintClaimCoreAccs<bool> = MintClaimCoreAccs::memset(false);

// Data

pub const CLAIM_IX_DISCM: u8 = 3;

pub const CLAIM_IX_DATA_LEN: usize = ClaimIxData::LEN;

pub type ClaimIxData = U64OptIxData<CLAIM_IX_DISCM>;

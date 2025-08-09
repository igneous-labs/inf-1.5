use inf1_core::inf1_ctl_core::pda::{pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds};
use inf1_svc_ag_std::inf1_svc_spl_core::sanctum_spl_stake_pool_core::{
    ASSOCIATED_TOKEN_PROGRAM, TOKEN_PROGRAM,
};

use crate::Inf;

#[inline]
pub fn find_pool_reserves_ata(
    find_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
    mint: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let [s1, s2, s3] = pool_reserves_ata_seeds(&TOKEN_PROGRAM, mint);
    find_pda(&[s1, s2, s3], &ASSOCIATED_TOKEN_PROGRAM)
}

#[inline]
pub fn find_protocol_fee_accumulator_ata(
    find_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
    mint: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let [s1, s2, s3] = protocol_fee_accumulator_ata_seeds(&TOKEN_PROGRAM, mint);
    find_pda(&[s1, s2, s3], &ASSOCIATED_TOKEN_PROGRAM)
}

#[inline]
pub fn create_pool_reserves_ata(
    create_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    mint: &[u8; 32],
    bump: u8,
) -> Option<[u8; 32]> {
    let [s1, s2, s3] = pool_reserves_ata_seeds(&TOKEN_PROGRAM, mint);
    create_pda(&[s1.as_slice(), s2, s3, &[bump]], &ASSOCIATED_TOKEN_PROGRAM)
}

#[inline]
pub fn create_protocol_fee_accumulator_ata(
    create_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    mint: &[u8; 32],
    bump: u8,
) -> Option<[u8; 32]> {
    let [s1, s2, s3] = protocol_fee_accumulator_ata_seeds(&TOKEN_PROGRAM, mint);
    // unwrap-safety: seeds are within range
    create_pda(&[s1.as_slice(), s2, s3, &[bump]], &ASSOCIATED_TOKEN_PROGRAM)
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>, C> Inf<F, C> {
    #[inline]
    pub fn find_pool_reserves_ata(&self, mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
        find_pool_reserves_ata(&self.find_pda, mint)
    }

    #[inline]
    pub fn find_protocol_fee_accumulator_ata(&self, mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
        find_protocol_fee_accumulator_ata(&self.find_pda, mint)
    }
}

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn create_pool_reserves_ata(&self, mint: &[u8; 32], bump: u8) -> Option<[u8; 32]> {
        create_pool_reserves_ata(&self.create_pda, mint, bump)
    }

    #[inline]
    pub fn create_protocol_fee_accumulator_ata(
        &self,
        mint: &[u8; 32],
        bump: u8,
    ) -> Option<[u8; 32]> {
        create_protocol_fee_accumulator_ata(&self.create_pda, mint, bump)
    }
}

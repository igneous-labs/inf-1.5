use inf1_core::inf1_ctl_core::token_info::TokenInfo;

use inf1_svc_ag_std::inf1_svc_spl_core::sanctum_spl_stake_pool_core::ASSOCIATED_TOKEN_PROGRAM;

use crate::Inf;

// Re-exports
pub use inf1_core::inf1_ctl_core::pda::*;

/// - `auth`: POOL_STATE for pool reserves, PROTOCOL_FEE for protocol fee accumulator
#[inline]
pub fn find_ata(
    find_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
    auth: &[u8; 32],
    token: &TokenInfo<&[u8; 32]>,
) -> Option<([u8; 32], u8)> {
    let [s1, s2, s3] = ata_seeds(auth, token);
    find_pda(&[s1, s2, s3], &ASSOCIATED_TOKEN_PROGRAM)
}

#[inline]
pub fn find_pool_reserves_ata(
    find_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
    token: &TokenInfo<&[u8; 32]>,
) -> Option<([u8; 32], u8)> {
    find_ata(find_pda, CONST_PDA_KEYS_OWNED.pool_state(), token)
}

#[inline]
pub fn find_protocol_fee_accumulator_ata(
    find_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
    token: &TokenInfo<&[u8; 32]>,
) -> Option<([u8; 32], u8)> {
    find_ata(find_pda, CONST_PDA_KEYS_OWNED.protocol_fee(), token)
}

/// - `auth`: POOL_STATE for pool reserves, PROTOCOL_FEE for protocol fee accumulator
#[inline]
pub fn create_ata(
    create_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    auth: &[u8; 32],
    token: &TokenInfo<&[u8; 32]>,
    bump: u8,
) -> Option<[u8; 32]> {
    let [s1, s2, s3] = ata_seeds(auth, token);
    create_pda(&[s1, s2, s3, &[bump]], &ASSOCIATED_TOKEN_PROGRAM)
}

#[inline]
pub fn create_pool_reserves_ata(
    create_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    token: &TokenInfo<&[u8; 32]>,
    bump: u8,
) -> Option<[u8; 32]> {
    create_ata(create_pda, CONST_PDA_KEYS_OWNED.pool_state(), token, bump)
}

#[inline]
pub fn create_protocol_fee_accumulator_ata(
    create_pda: impl FnOnce(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    token: &TokenInfo<&[u8; 32]>,
    bump: u8,
) -> Option<[u8; 32]> {
    create_ata(create_pda, CONST_PDA_KEYS_OWNED.protocol_fee(), token, bump)
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>, C> Inf<F, C> {
    #[inline]
    pub fn find_pool_reserves_ata(&self, token: &TokenInfo<&[u8; 32]>) -> Option<([u8; 32], u8)> {
        find_pool_reserves_ata(&self.find_pda, token)
    }

    #[inline]
    pub fn find_protocol_fee_accumulator_ata(
        &self,
        token: &TokenInfo<&[u8; 32]>,
    ) -> Option<([u8; 32], u8)> {
        find_protocol_fee_accumulator_ata(&self.find_pda, token)
    }
}

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn create_pool_reserves_ata(
        &self,
        token: &TokenInfo<&[u8; 32]>,
        bump: u8,
    ) -> Option<[u8; 32]> {
        create_pool_reserves_ata(&self.create_pda, token, bump)
    }

    #[inline]
    pub fn create_protocol_fee_accumulator_ata(
        &self,
        token: &TokenInfo<&[u8; 32]>,
        bump: u8,
    ) -> Option<[u8; 32]> {
        create_protocol_fee_accumulator_ata(&self.create_pda, token, bump)
    }
}

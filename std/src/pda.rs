use inf1_core::inf1_ctl_core::{keys::CONST_KEYS_OWNED, token_info::TokenInfo};

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
    find_pda(&[s1, s2, s3], CONST_KEYS_OWNED.atoken())
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
    create_pda(&[s1, s2, s3, &[bump]], CONST_KEYS_OWNED.atoken())
}

impl<F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>, C> Inf<F, C> {
    #[inline]
    fn find_const_pda(
        &self,
        get_default: impl Fn(&ConstPdas<([u8; 32], u8)>) -> &([u8; 32], u8),
        get_cached: impl Fn(&ConstPdas<Option<([u8; 32], u8)>>) -> &Option<([u8; 32], u8)>,
        seeds: &[&[u8]],
    ) -> Option<([u8; 32], u8)> {
        self.prog.as_ref().map_or_else(
            || Some(*get_default(&CONST_PDAS)),
            |ProgAddrs {
                 prog_id,
                 const_pda_cache,
             }| {
                get_cached(const_pda_cache).or_else(|| (self.find_pda)(seeds, prog_id))
            },
        )
    }

    #[inline]
    pub fn find_pool_state(&self) -> Option<([u8; 32], u8)> {
        self.find_const_pda(
            ConstPdas::pool_state,
            ConstPdas::pool_state,
            &[&POOL_STATE_SEED],
        )
    }

    #[inline]
    pub fn find_lst_state_list(&self) -> Option<([u8; 32], u8)> {
        self.find_const_pda(
            ConstPdas::lst_state_list,
            ConstPdas::lst_state_list,
            &[&LST_STATE_LIST_SEED],
        )
    }

    #[inline]
    pub fn find_protocol_fee(&self) -> Option<([u8; 32], u8)> {
        self.find_const_pda(
            ConstPdas::protocol_fee,
            ConstPdas::protocol_fee,
            &[&PROTOCOL_FEE_SEED],
        )
    }

    #[inline]
    pub fn find_rebalance_record(&self) -> Option<([u8; 32], u8)> {
        self.find_const_pda(
            ConstPdas::rebalance_record,
            ConstPdas::rebalance_record,
            &[&REBALANCE_RECORD_SEED],
        )
    }

    // TODO: disable pool authority list

    #[inline]
    fn find_cache_const_pda(
        &mut self,
        get_default: impl Fn(&ConstPdas<([u8; 32], u8)>) -> &([u8; 32], u8),
        get_cached: impl Fn(&ConstPdas<Option<([u8; 32], u8)>>) -> &Option<([u8; 32], u8)>,
        seeds: &[&[u8]],
        mut set_cached: impl FnMut(
            &mut ConstPdas<Option<([u8; 32], u8)>>,
            Option<([u8; 32], u8)>,
        ) -> Option<([u8; 32], u8)>,
    ) -> Option<([u8; 32], u8)> {
        self.prog.as_mut().map_or_else(
            || Some(*get_default(&CONST_PDAS)),
            |ProgAddrs {
                 prog_id,
                 const_pda_cache,
             }| {
                get_cached(const_pda_cache).or_else(|| {
                    let res = Some((self.find_pda)(seeds, prog_id)?);
                    set_cached(const_pda_cache, res);
                    res
                })
            },
        )
    }

    #[inline]
    pub fn find_cache_pool_state(&mut self) -> Option<([u8; 32], u8)> {
        self.find_cache_const_pda(
            ConstPdas::pool_state,
            ConstPdas::pool_state,
            &[&POOL_STATE_SEED],
            ConstPdas::set_pool_state,
        )
    }

    #[inline]
    pub fn find_cache_lst_state_list(&mut self) -> Option<([u8; 32], u8)> {
        self.find_cache_const_pda(
            ConstPdas::lst_state_list,
            ConstPdas::lst_state_list,
            &[&LST_STATE_LIST_SEED],
            ConstPdas::set_lst_state_list,
        )
    }

    #[inline]
    pub fn find_cache_protocol_fee(&mut self) -> Option<([u8; 32], u8)> {
        self.find_cache_const_pda(
            ConstPdas::protocol_fee,
            ConstPdas::protocol_fee,
            &[&PROTOCOL_FEE_SEED],
            ConstPdas::set_protocol_fee,
        )
    }

    #[inline]
    pub fn find_cache_rebalance_record(&mut self) -> Option<([u8; 32], u8)> {
        self.find_cache_const_pda(
            ConstPdas::rebalance_record,
            ConstPdas::rebalance_record,
            &[&REBALANCE_RECORD_SEED],
            ConstPdas::set_rebalance_record,
        )
    }

    // TODO: disable pool authority list

    #[inline]
    pub fn find_pool_reserves_ata(&self, token: &TokenInfo<&[u8; 32]>) -> Option<([u8; 32], u8)> {
        find_ata(&self.find_pda, &self.find_pool_state()?.0, token)
    }

    /// Also caches pool state const PDA if applicable
    #[inline]
    pub fn find_pool_reserves_ata_mut(
        &mut self,
        token: &TokenInfo<&[u8; 32]>,
    ) -> Option<([u8; 32], u8)> {
        let auth = self.find_cache_pool_state()?.0;
        find_ata(&self.find_pda, &auth, token)
    }

    /// Also caches protocol fee const PDA if applicable
    #[inline]
    pub fn find_protocol_fee_accumulator_ata_mut(
        &mut self,
        token: &TokenInfo<&[u8; 32]>,
    ) -> Option<([u8; 32], u8)> {
        let auth = self.find_cache_protocol_fee()?.0;
        find_ata(&self.find_pda, &auth, token)
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    #[inline]
    pub fn create_pool_reserves_ata(
        &self,
        token: &TokenInfo<&[u8; 32]>,
        bump: u8,
    ) -> Option<[u8; 32]> {
        create_ata(&self.create_pda, &self.find_pool_state()?.0, token, bump)
    }

    #[inline]
    pub fn create_protocol_fee_accumulator_ata(
        &self,
        token: &TokenInfo<&[u8; 32]>,
        bump: u8,
    ) -> Option<[u8; 32]> {
        create_ata(&self.create_pda, &self.find_protocol_fee()?.0, token, bump)
    }

    /// Also caches pool state const PDA if applicable
    #[inline]
    pub fn create_pool_reserves_ata_mut(
        &mut self,
        token: &TokenInfo<&[u8; 32]>,
        bump: u8,
    ) -> Option<[u8; 32]> {
        let auth = self.find_cache_pool_state()?.0;
        create_ata(&self.create_pda, &auth, token, bump)
    }

    /// Also caches protocol fee const PDA if applicable
    #[inline]
    pub fn create_protocol_fee_accumulator_ata_mut(
        &mut self,
        token: &TokenInfo<&[u8; 32]>,
        bump: u8,
    ) -> Option<[u8; 32]> {
        let auth = self.find_cache_protocol_fee()?.0;
        create_ata(&self.create_pda, &auth, token, bump)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProgAddrs {
    pub prog_id: [u8; 32],

    /// write-once cache, private field guarantees
    const_pda_cache: ConstPdas<Option<([u8; 32], u8)>>,
}

impl ProgAddrs {
    #[inline]
    pub const fn new(prog_id: [u8; 32]) -> Self {
        Self {
            prog_id,
            const_pda_cache: ConstPdas([None; _]),
        }
    }
}

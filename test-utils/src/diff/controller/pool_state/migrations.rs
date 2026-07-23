use inf1_ctl_core::accounts::pool_state::{PoolStateV2, VerPoolState};
use solana_account::Account;

use crate::{
    assert_diffs_pool_state_v2, pool_state_account_for_migration, pool_state_v2_account,
    DiffsPoolStateV2,
};

pub fn ver_pool_state_into_account(p: VerPoolState) -> Account {
    match p {
        VerPoolState::V1(p) => pool_state_account_for_migration(p),
        VerPoolState::V2(p) => pool_state_v2_account(p),
    }
}

/// _nm = "no migration"
pub fn assert_diffs_pool_state_nm(diffs: DiffsPoolStateV2, bef: &VerPoolState, aft: &PoolStateV2) {
    match bef {
        VerPoolState::V2(bef) => assert_diffs_pool_state_v2(&diffs, bef, aft),
        VerPoolState::V1(_) => panic!("unexpected v1 -> v2 migration"),
    };
}

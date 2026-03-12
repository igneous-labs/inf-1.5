use inf1_ctl_core::{
    accounts::pool_state::{migrated_protocol_fee_nanos, PoolStateV2, VerPoolState},
    typedefs::fee_nanos::FeeNanos,
};
use solana_account::Account;

use crate::{
    assert_diffs_pool_state_v2, pool_state_account_for_migration, pool_state_v2_account, Diff,
    DiffsPoolStateV2,
};

pub fn ver_pool_state_into_account(p: VerPoolState) -> Account {
    match p {
        VerPoolState::V1(p) => pool_state_account_for_migration(p),
        VerPoolState::V2(p) => pool_state_v2_account(p),
    }
}

/// _mm = "maybe migration"
pub fn assert_diffs_pool_state_mm(
    mut diffs: DiffsPoolStateV2,
    bef: &VerPoolState,
    aft: &PoolStateV2,
    migration_slot: u64,
) {
    match bef {
        VerPoolState::V2(bef) => assert_diffs_pool_state_v2(&diffs, bef, aft),
        VerPoolState::V1(bef_v1) => {
            diffs
                .addrs
                .set_rps_authority(diff_for_migration(*diffs.addrs.rps_authority(), || {
                    bef_v1.admin
                }));
            diffs.rps = diff_for_migration(diffs.rps, Default::default);
            diffs.protocol_fee_nanos = diff_for_migration(diffs.protocol_fee_nanos, || {
                FeeNanos::new(migrated_protocol_fee_nanos(
                    bef_v1.lp_protocol_fee_bps,
                    bef_v1.trading_protocol_fee_bps,
                ))
                .unwrap()
            });
            diffs
                .u64s
                .set_last_release_slot(diff_for_migration(*diffs.u64s.last_release_slot(), || {
                    migration_slot
                }));

            // assert vers diff separately since .migrated() changes it
            Diff::StrictChanged(1, 2).assert(&bef_v1.version, &aft.version);

            assert_diffs_pool_state_v2(&diffs, &bef.migrated(migration_slot), aft);
        }
    };
}

fn diff_for_migration<T: Clone + Default>(
    diff: Diff<T>,
    gen_expected_aft_migration: impl FnOnce() -> T,
) -> Diff<T> {
    match &diff {
        // For lenient diffs, check against expected aft migration
        Diff::Pass | Diff::Unchanged => {
            let new = gen_expected_aft_migration();
            Diff::Changed(diff.passable_old(&new), new)
        }
        Diff::Changed(_, new) => Diff::Changed(diff.passable_old(new), new.clone()),
        Diff::StrictChanged(_, new) => Diff::StrictChanged(diff.passable_old(new), new.clone()),
        Diff::GreaterOrEqual(_) => diff,
    }
}

use inf1_ctl_core::{
    accounts::pool_state::{PoolState, PoolStatePacked, PoolStateV2, PoolStateV2Packed},
    typedefs::{fee_nanos::FeeNanos, rps::Rps},
};
use solana_account::Account;

use crate::{
    assert_diffs_pool_state_v2, pool_state_account, pool_state_v2_account, Diff, DiffsPoolStateV2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerPS<V1, V2> {
    V1(V1),
    V2(V2),
}

pub type VerPoolState = VerPS<PoolState, PoolStateV2>;

macro_rules! each_variant_field {
    ($ag:expr, $field:ident) => {
        match $ag {
            VerPS::V1(p) => p.$field,
            VerPS::V2(p) => p.$field,
        }
    };
}

impl VerPoolState {
    pub fn from_acc_data(data: &[u8]) -> Self {
        if let Some(p) = PoolStatePacked::of_acc_data(data) {
            Self::V1(p.into_pool_state())
        } else {
            Self::V2(
                PoolStateV2Packed::of_acc_data(data)
                    .unwrap()
                    .into_pool_state_v2(),
            )
        }
    }

    pub fn into_account(self) -> Account {
        match self {
            Self::V1(p) => pool_state_account(p),
            Self::V2(p) => pool_state_v2_account(p),
        }
    }

    pub fn total_sol_value(&self) -> u64 {
        each_variant_field!(self, total_sol_value)
    }

    pub fn migrated(self, migration_slot: u64) -> PoolStateV2 {
        match self {
            Self::V2(p) => p,
            Self::V1(PoolState {
                total_sol_value,
                trading_protocol_fee_bps,
                lp_protocol_fee_bps,
                is_disabled,
                is_rebalancing,
                padding,
                admin,
                rebalance_authority,
                protocol_fee_beneficiary,
                pricing_program,
                lp_token_mint,
                version: _,
            }) => PoolStateV2 {
                total_sol_value,
                is_disabled,
                is_rebalancing,
                padding,
                admin,
                rebalance_authority,
                protocol_fee_beneficiary,
                pricing_program,
                lp_token_mint,
                // kinda dumb reimplementing the same logic in the program here again but
                // serves as double-check i guess
                protocol_fee_nanos: *migrated_protocol_fee_nanos(
                    lp_protocol_fee_bps,
                    trading_protocol_fee_bps,
                ),
                rps: *Rps::default().as_raw(),
                rps_authority: admin,
                last_release_slot: migration_slot,
                version: 2,
                withheld_lamports: 0,
                protocol_fee_lamports: 0,
            },
        }
    }
}

// kinda dumb reimplementing the same logic in the program here again but
// serves as double-check i guess
fn migrated_protocol_fee_nanos(
    lp_protocol_fee_bps: u16,
    trading_protocol_fee_bps: u16,
) -> FeeNanos {
    FeeNanos::new(
        u32::from(
            [lp_protocol_fee_bps, trading_protocol_fee_bps]
                .into_iter()
                .max()
                .unwrap(),
        ) * 100_000,
    )
    .unwrap()
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
                migrated_protocol_fee_nanos(
                    bef_v1.lp_protocol_fee_bps,
                    bef_v1.trading_protocol_fee_bps,
                )
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

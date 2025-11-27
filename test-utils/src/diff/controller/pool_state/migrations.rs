use inf1_ctl_core::{
    accounts::pool_state::{
        NewPoolStateV2AddrsBuilder, NewPoolStateV2U8BoolsBuilder, PoolState, PoolStatePacked,
        PoolStateV2, PoolStateV2FtaVals, PoolStateV2Packed, PoolStateV2U8Bools,
    },
    typedefs::fee_nanos::FeeNanos,
};
use solana_account::Account;

use crate::{
    assert_diffs_pool_state_v2, gas_diff_zip_assert, pool_state_account, pool_state_to_gen_args,
    pool_state_v2_account, u8_to_bool, Diff, DiffsPoolStateV2, GenPoolStateArgs,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerPS<V1, V2> {
    V1(V1),
    V2(V2),
}

pub type VerPoolState = VerPS<PoolState, PoolStateV2>;

macro_rules! map_variant_field {
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
        map_variant_field!(self, total_sol_value)
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
        VerPoolState::V1(bef) => {
            diffs
                .addrs
                .set_rps_authority(migration_strict(*diffs.addrs.rps_authority(), || bef.admin));
            diffs.rps = migration_strict(diffs.rps, Default::default);
            diffs.protocol_fee_nanos = migration_strict(diffs.protocol_fee_nanos, || {
                // kinda dumb reimplementing the same logic in the program here again but
                // serves as double-check i guess
                FeeNanos::new(
                    u32::from(
                        [bef.lp_protocol_fee_bps, bef.trading_protocol_fee_bps]
                            .into_iter()
                            .max()
                            .unwrap(),
                    ) * 100_000,
                )
                .unwrap()
            });
            diffs
                .u64s
                .set_last_release_slot(migration_strict(*diffs.u64s.last_release_slot(), || {
                    migration_slot
                }));

            assert_diffs_pool_state_v1_v2(&diffs, bef, aft);
        }
    };
}

/// If the given Diff is lenient, make it stricter by setting it to expected
/// migration change defaults.
///
/// Otherwise, passthrough
fn migration_strict<T: Default>(
    diff: Diff<T>,
    gen_expected_aft_migration: impl FnOnce() -> T,
) -> Diff<T> {
    match diff {
        Diff::Pass | Diff::Unchanged => {
            Diff::Changed(Default::default(), gen_expected_aft_migration())
        }
        Diff::Changed(..) | Diff::GreaterOrEqual(..) | Diff::StrictChanged(..) => diff,
    }
}

pub fn assert_diffs_pool_state_v1_v2(
    DiffsPoolStateV2 {
        addrs,
        u64s,
        u8_bools,
        protocol_fee_nanos,
        rps,
    }: &DiffsPoolStateV2,
    bef: &PoolState,
    aft: &PoolStateV2,
) {
    assert_eq!(bef.version, 1);
    assert_eq!(aft.version, 2);

    let GenPoolStateArgs {
        total_sol_value: bef_total_sol_value,
        bools: bef_bools,
        pks: bef_pks,
        ..
    }: GenPoolStateArgs = pool_state_to_gen_args(bef);
    let PoolStateV2FtaVals {
        addrs: aft_addrs,
        u64s: aft_u64s,
        u8_bools: aft_u8_bools,
        protocol_fee_nanos: aft_protocol_fee_nanos,
        rps: aft_rps,
    } = PoolStateV2FtaVals::try_from_pool_state_v2(*aft).unwrap();

    let bef_addrs = NewPoolStateV2AddrsBuilder::start()
        .with_admin(*bef_pks.admin())
        .with_lp_token_mint(*bef_pks.lp_token_mint())
        .with_pricing_program(*bef_pks.pricing_program())
        .with_protocol_fee_beneficiary(*bef_pks.protocol_fee_beneficiary())
        .with_rebalance_authority(*bef_pks.rebalance_authority())
        // newly-added
        .with_rps_authority(
            addrs
                .rps_authority()
                .passable_old(aft_addrs.rps_authority()),
        )
        .build();
    gas_diff_zip_assert!(addrs, bef_addrs, aft_addrs);

    let bef_u8_bools = NewPoolStateV2U8BoolsBuilder::start()
        .with_is_disabled(*bef_bools.is_disabled())
        .with_is_rebalancing(*bef_bools.is_rebalancing())
        .build();
    let aft_u8_bools = PoolStateV2U8Bools(aft_u8_bools.0.map(u8_to_bool));
    gas_diff_zip_assert!(u8_bools, bef_u8_bools, aft_u8_bools);

    u64s.total_sol_value()
        .assert(&bef_total_sol_value, aft_u64s.total_sol_value());

    // newly-added
    u64s.last_release_slot().assert(
        &u64s
            .last_release_slot()
            .passable_old(aft_u64s.last_release_slot()),
        aft_u64s.last_release_slot(),
    );
    protocol_fee_nanos.assert(
        &protocol_fee_nanos.passable_old(&aft_protocol_fee_nanos),
        &aft_protocol_fee_nanos,
    );
    rps.assert(&rps.passable_old(&aft_rps), &aft_rps);
}

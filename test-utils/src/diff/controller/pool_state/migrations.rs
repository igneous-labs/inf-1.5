use inf1_ctl_core::{
    accounts::pool_state::{
        NewPoolStateV2AddrsBuilder, NewPoolStateV2U8BoolsBuilder, PoolState, PoolStateV2,
        PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2U64s, PoolStateV2U8Bools,
    },
    typedefs::fee_nanos::FeeNanos,
};
use solana_clock::Clock;

use crate::{
    gas_diff_zip_assert, pool_state_to_gen_args, u8_to_bool, Diff, DiffsPoolStateV2,
    GenPoolStateArgs, NewPoolStateU16sBuilder, PoolStateU16s,
};

pub fn default_pool_state_migration_diffs_v1_v2(
    pool_state_bef: &PoolState,
    clock: &Clock,
) -> DiffsPoolStateV2 {
    default_pool_state_migration_diffs_v1_v2_inner(
        NewPoolStateU16sBuilder::start()
            .with_lp_protocol_fee_bps(pool_state_bef.lp_protocol_fee_bps)
            .with_trading_protocol_fee_bps(pool_state_bef.trading_protocol_fee_bps)
            .build(),
        pool_state_bef.admin,
        clock.slot,
    )
}

fn default_pool_state_migration_diffs_v1_v2_inner(
    u16s: PoolStateU16s<u16>,
    expected_rps_auth: [u8; 32],
    migration_slot: u64,
) -> DiffsPoolStateV2 {
    // kinda dumb reimplementing the same logic in the program here again but
    // serves as double-check i guess
    let expected_pf_nanos =
        FeeNanos::new(u32::from(u16s.0.into_iter().max().unwrap()) * 100_000).unwrap();

    DiffsPoolStateV2 {
        // use Changed + passable_old below to assert
        // that new fields are set to their correct default values.
        //
        // dont care about value of `Changed.0` / old here
        addrs: PoolStateV2Addrs::default()
            .with_rps_authority(Diff::Changed(Default::default(), expected_rps_auth)),
        rps: Diff::Changed(Default::default(), Default::default()),
        protocol_fee_nanos: Diff::Changed(Default::default(), expected_pf_nanos),
        u64s: PoolStateV2U64s::default()
            .with_last_release_slot(Diff::Changed(Default::default(), migration_slot)),
        ..Default::default()
    }
}

pub fn assert_pool_state_migration_v1_v2(
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

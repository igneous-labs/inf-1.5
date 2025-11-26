use inf1_ctl_core::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Fta, PoolStateV2FtaVals, PoolStateV2U8Bools},
    typedefs::{fee_nanos::FeeNanos, rps::Rps},
};

use crate::{gas_diff_zip_assert, u8_to_bool, Diff};

pub type DiffsPoolStateV2 =
    PoolStateV2Fta<Diff<[u8; 32]>, Diff<u64>, Diff<bool>, Diff<FeeNanos>, Diff<Rps>>;

pub fn assert_diffs_pool_state_v2(
    DiffsPoolStateV2 {
        addrs,
        u64s,
        u8_bools,
        protocol_fee_nanos,
        rps,
    }: &DiffsPoolStateV2,
    bef: &PoolStateV2,
    aft: &PoolStateV2,
) {
    let [PoolStateV2FtaVals {
        addrs: bef_addrs,
        u64s: bef_u64s,
        u8_bools: bef_u8_bools,
        protocol_fee_nanos: bef_protocol_fee_nanos,
        rps: bef_rps,
    }, PoolStateV2FtaVals {
        addrs: aft_addrs,
        u64s: aft_u64s,
        u8_bools: aft_u8_bools,
        protocol_fee_nanos: aft_protocol_fee_nanos,
        rps: aft_rps,
    }] = [bef, aft].map(|p| PoolStateV2FtaVals::try_from_pool_state_v2(*p).unwrap());

    let [bef_u8_bools, aft_u8_bools] =
        [bef_u8_bools, aft_u8_bools].map(|g| PoolStateV2U8Bools(g.0.map(u8_to_bool)));

    gas_diff_zip_assert!(addrs, bef_addrs, aft_addrs);
    gas_diff_zip_assert!(u64s, bef_u64s, aft_u64s);
    gas_diff_zip_assert!(u8_bools, bef_u8_bools, aft_u8_bools);
    protocol_fee_nanos.assert(&bef_protocol_fee_nanos, &aft_protocol_fee_nanos);
    rps.assert(&bef_rps, &aft_rps);
}

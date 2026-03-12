use inf1_ctl_core::accounts::pool_state::PoolState;

use crate::{
    gas_diff_zip_assert, pool_state_to_gen_args, Diff, GenPoolStateArgs, PoolStateArgs,
    PoolStateBools, PoolStatePks, PoolStateU16s,
};

pub type DiffsPoolStateArgs = PoolStateArgs<
    Diff<u64>,
    PoolStateU16s<Diff<u16>>,
    Diff<u8>,
    PoolStateBools<Diff<bool>>,
    PoolStatePks<Diff<[u8; 32]>>,
>;

pub fn assert_diffs_pool_state(
    DiffsPoolStateArgs {
        total_sol_value,
        u16s,
        version,
        bools,
        pks,
    }: &DiffsPoolStateArgs,
    bef: &PoolState,
    aft: &PoolState,
) {
    let [GenPoolStateArgs {
        total_sol_value: bef_total_sol_value,
        u16s: bef_u16s,
        version: bef_version,
        bools: bef_bools,
        pks: bef_pks,
    }, GenPoolStateArgs {
        total_sol_value: aft_total_sol_value,
        u16s: aft_u16s,
        version: aft_version,
        bools: aft_bools,
        pks: aft_pks,
    }] = [bef, aft].map(pool_state_to_gen_args);
    total_sol_value.assert(&bef_total_sol_value, &aft_total_sol_value);
    gas_diff_zip_assert!(u16s, bef_u16s, aft_u16s);
    version.assert(&bef_version, &aft_version);
    gas_diff_zip_assert!(bools, bef_bools, aft_bools);
    gas_diff_zip_assert!(pks, bef_pks, aft_pks);
}

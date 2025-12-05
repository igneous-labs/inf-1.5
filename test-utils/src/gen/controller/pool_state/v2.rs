use inf1_ctl_core::{
    accounts::pool_state::{
        NewPoolStateV2U8BoolsBuilder, PoolStateV2, PoolStateV2Addrs, PoolStateV2Fta,
        PoolStateV2FtaVals, PoolStateV2U64s, PoolStateV2U8Bools,
    },
    typedefs::{
        fee_nanos::FeeNanos,
        pool_sv::{NewPoolSvBuilder, PoolSvLamports},
        rps::Rps,
    },
};
use jiminy_sysvar_rent::Rent;
use proptest::prelude::*;
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::{
    any_ctl_fee_nanos_strat, any_rps_strat, bals_from_supply, bool_strat, bool_to_u8, pk_strat,
    u64_strat,
};

/// If `Option::None`, `any()` is used
pub type PoolStateV2FtaStrat = PoolStateV2Fta<
    Option<BoxedStrategy<[u8; 32]>>,
    Option<BoxedStrategy<u64>>,
    Option<BoxedStrategy<bool>>,
    Option<BoxedStrategy<FeeNanos>>,
    Option<BoxedStrategy<Rps>>,
>;

/// Not disabled, not rebalancing
pub fn pool_state_v2_u8_bools_normal_strat() -> PoolStateV2U8Bools<Option<BoxedStrategy<bool>>> {
    PoolStateV2U8Bools(
        NewPoolStateV2U8BoolsBuilder::start()
            .with_is_disabled(false)
            .with_is_rebalancing(false)
            .build()
            .0
            .map(|x| Some(Just(x).boxed())),
    )
}

pub fn pool_sv_lamports_solvent_strat(tsv: u64) -> impl Strategy<Value = PoolSvLamports> {
    bals_from_supply(tsv).prop_map(move |([withheld, protocol_fee], _rem)| {
        NewPoolSvBuilder::start()
            .with_total(tsv)
            .with_withheld(withheld)
            .with_protocol_fee(protocol_fee)
            .build()
    })
}

pub fn any_pool_sv_lamports_solvent_strat() -> impl Strategy<Value = PoolSvLamports> {
    (0..=u64::MAX).prop_flat_map(pool_sv_lamports_solvent_strat)
}

pub fn pool_state_v2_u64s_with_just_lamports_strat(
    this: PoolStateV2U64s<Option<BoxedStrategy<u64>>>,
    x: PoolSvLamports,
) -> PoolStateV2U64s<Option<BoxedStrategy<u64>>> {
    let to_sjb = |x: &u64| Some(Just(*x).boxed());
    this.with_total_sol_value(to_sjb(x.total()))
        .with_withheld_lamports(to_sjb(x.withheld()))
        .with_protocol_fee_lamports(to_sjb(x.protocol_fee()))
}

pub fn pool_state_v2_u64s_just_lamports_strat(
    x: PoolSvLamports,
) -> PoolStateV2U64s<Option<BoxedStrategy<u64>>> {
    pool_state_v2_u64s_with_just_lamports_strat(PoolStateV2U64s::default(), x)
}

pub fn pool_state_v2_u64s_with_last_release_slot_bef_incl(
    this: PoolStateV2U64s<Option<BoxedStrategy<u64>>>,
    bef_incl: u64,
) -> PoolStateV2U64s<Option<BoxedStrategy<u64>>> {
    this.with_last_release_slot(Some((0..=bef_incl).boxed()))
}

pub fn any_pool_state_v2(
    PoolStateV2FtaStrat {
        addrs,
        u64s,
        u8_bools,
        protocol_fee_nanos,
        rps,
    }: PoolStateV2FtaStrat,
) -> impl Strategy<Value = PoolStateV2> {
    let u64s = u64s.0.map(u64_strat);
    let bools = u8_bools.0.map(bool_strat);
    let addrs = addrs.0.map(pk_strat);
    let protocol_fee_nanos =
        protocol_fee_nanos.unwrap_or_else(|| any_ctl_fee_nanos_strat().boxed());
    let rps = rps.unwrap_or_else(|| any_rps_strat().boxed());
    (u64s, bools, addrs, protocol_fee_nanos, rps).prop_map(
        |(u64s, bools, addrs, protocol_fee_nanos, rps)| {
            PoolStateV2FtaVals {
                addrs: PoolStateV2Addrs(addrs),
                u64s: PoolStateV2U64s(u64s),
                u8_bools: PoolStateV2U8Bools(bools.map(bool_to_u8)),
                protocol_fee_nanos,
                rps,
            }
            .into_pool_state_v2()
        },
    )
}

pub fn pool_state_v2_account(data: PoolStateV2) -> Account {
    Account {
        lamports: Rent::DEFAULT.min_balance(data.as_acc_data_arr().len()),
        data: data.as_acc_data_arr().into(),
        owner: Pubkey::new_from_array(inf1_ctl_core::ID),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

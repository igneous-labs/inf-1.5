use generic_array_struct::generic_array_struct;
use inf1_ctl_core::accounts::pool_state::PoolState;
use jiminy_sysvar_rent::Rent;
use proptest::prelude::*;
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::{bool_strat, bool_to_u8, pk_strat, u16_strat, u64_strat};

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStatePks<T> {
    pub admin: T,
    pub rebalance_authority: T,
    pub protocol_fee_beneficiary: T,
    pub pricing_program: T,
    pub lp_token_mint: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStateU16s<T> {
    pub trading_protocol_fee_bps: T,
    pub lp_protocol_fee_bps: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStateBools<T> {
    pub is_disabled: T,
    pub is_rebalancing: T,
}

impl PoolStateBools<Option<BoxedStrategy<bool>>> {
    /// Not disabled, not rebalancing
    pub fn normal() -> Self {
        Self(
            NewPoolStateBoolsBuilder::start()
                .with_is_disabled(false)
                .with_is_rebalancing(false)
                .build()
                .0
                .map(|x| Some(Just(x).boxed())),
        )
    }
}

pub fn gen_pool_state(
    total_sol_value: u64,
    u16s: PoolStateU16s<u16>,
    version: u8,
    bools: PoolStateBools<bool>,
    pks: PoolStatePks<[u8; 32]>,
) -> PoolState {
    let bools = PoolStateBools(bools.0.map(bool_to_u8));
    PoolState {
        total_sol_value,
        trading_protocol_fee_bps: *u16s.trading_protocol_fee_bps(),
        lp_protocol_fee_bps: *u16s.lp_protocol_fee_bps(),
        version,
        is_disabled: *bools.is_disabled(),
        is_rebalancing: *bools.is_rebalancing(),
        padding: [0],
        admin: *pks.admin(),
        rebalance_authority: *pks.rebalance_authority(),
        protocol_fee_beneficiary: *pks.protocol_fee_beneficiary(),
        pricing_program: *pks.pricing_program(),
        lp_token_mint: *pks.lp_token_mint(),
    }
}

/// If `Option::None`, `any()` is used
#[derive(Debug, Clone, Default)]
pub struct GenPoolStateArgs {
    pub total_sol_value: Option<BoxedStrategy<u64>>,
    pub u16s: PoolStateU16s<Option<BoxedStrategy<u16>>>,
    pub version: Option<BoxedStrategy<u8>>,
    pub bools: PoolStateBools<Option<BoxedStrategy<bool>>>,
    pub pks: PoolStatePks<Option<BoxedStrategy<[u8; 32]>>>,
}

pub fn any_pool_state(
    GenPoolStateArgs {
        total_sol_value,
        u16s,
        version,
        bools,
        pks,
    }: GenPoolStateArgs,
) -> impl Strategy<Value = PoolState> {
    let total_sol_value = u64_strat(total_sol_value);
    let u16s = u16s.0.map(u16_strat);
    let version = version.unwrap_or_else(|| Just(1).boxed());
    let bools = bools.0.map(bool_strat);
    let pks = pks.0.map(pk_strat);
    (total_sol_value, u16s, version, bools, pks).prop_map(
        |(total_sol_value, u16s, version, bools, pks)| {
            gen_pool_state(
                total_sol_value,
                PoolStateU16s(u16s),
                version,
                PoolStateBools(bools),
                PoolStatePks(pks),
            )
        },
    )
}

pub fn pool_state_account(data: PoolState) -> Account {
    Account {
        lamports: Rent::DEFAULT.min_balance(data.as_acc_data_arr().len()),
        data: data.as_acc_data_arr().into(),
        owner: Pubkey::new_from_array(inf1_ctl_core::ID),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

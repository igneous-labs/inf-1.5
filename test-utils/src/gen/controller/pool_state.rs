use generic_array_struct::generic_array_struct;
use inf1_ctl_core::accounts::pool_state::PoolState;
use jiminy_sysvar_rent::Rent;
use proptest::prelude::*;
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::{
    bool_strat, bool_to_u8, gas_diff_zip_assert, pk_strat, u16_strat, u64_strat, u8_to_bool, Diff,
};

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

#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStateArgs<T, U, V, B, P> {
    pub total_sol_value: T,
    pub u16s: U,
    pub version: V,
    pub bools: B,
    pub pks: P,
}

pub type GenPoolStateArgs =
    PoolStateArgs<u64, PoolStateU16s<u16>, u8, PoolStateBools<bool>, PoolStatePks<[u8; 32]>>;

pub fn gen_pool_state(
    GenPoolStateArgs {
        total_sol_value,
        u16s,
        version,
        bools,
        pks,
    }: GenPoolStateArgs,
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
pub type AnyPoolStateArgs = PoolStateArgs<
    Option<BoxedStrategy<u64>>,
    PoolStateU16s<Option<BoxedStrategy<u16>>>,
    Option<BoxedStrategy<u8>>,
    PoolStateBools<Option<BoxedStrategy<bool>>>,
    PoolStatePks<Option<BoxedStrategy<[u8; 32]>>>,
>;

pub fn any_pool_state(
    AnyPoolStateArgs {
        total_sol_value,
        u16s,
        version,
        bools,
        pks,
    }: AnyPoolStateArgs,
) -> impl Strategy<Value = PoolState> {
    let total_sol_value = u64_strat(total_sol_value);
    let u16s = u16s.0.map(u16_strat);
    // currently defaults to v1, modify in future if needed
    let version = version.unwrap_or_else(|| Just(1).boxed());
    let bools = bools.0.map(bool_strat);
    let pks = pks.0.map(pk_strat);
    (total_sol_value, u16s, version, bools, pks).prop_map(
        |(total_sol_value, u16s, version, bools, pks)| {
            gen_pool_state(GenPoolStateArgs {
                total_sol_value,
                u16s: PoolStateU16s(u16s),
                version,
                bools: PoolStateBools(bools),
                pks: PoolStatePks(pks),
            })
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

pub type DiffsPoolStateArgs = PoolStateArgs<
    Diff<u64>,
    PoolStateU16s<Diff<u16>>,
    Diff<u8>,
    PoolStateBools<Diff<bool>>,
    PoolStatePks<Diff<[u8; 32]>>,
>;

fn pool_state_to_gen_args(
    PoolState {
        total_sol_value,
        trading_protocol_fee_bps,
        lp_protocol_fee_bps,
        version,
        is_disabled,
        is_rebalancing,
        admin,
        rebalance_authority,
        protocol_fee_beneficiary,
        pricing_program,
        lp_token_mint,
        padding: _,
    }: &PoolState,
) -> GenPoolStateArgs {
    GenPoolStateArgs {
        total_sol_value: *total_sol_value,
        u16s: NewPoolStateU16sBuilder::start()
            .with_lp_protocol_fee_bps(*lp_protocol_fee_bps)
            .with_trading_protocol_fee_bps(*trading_protocol_fee_bps)
            .build(),
        version: *version,
        bools: NewPoolStateBoolsBuilder::start()
            .with_is_disabled(u8_to_bool(*is_disabled))
            .with_is_rebalancing(u8_to_bool(*is_rebalancing))
            .build(),
        pks: NewPoolStatePksBuilder::start()
            .with_admin(*admin)
            .with_lp_token_mint(*lp_token_mint)
            .with_pricing_program(*pricing_program)
            .with_protocol_fee_beneficiary(*protocol_fee_beneficiary)
            .with_rebalance_authority(*rebalance_authority)
            .build(),
    }
}

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

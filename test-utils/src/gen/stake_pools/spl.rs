use generic_array_struct::generic_array_struct;
use proptest::{prelude::*, strategy::Union};
use sanctum_spl_stake_pool_core::{AccountType, Fee, FutureEpoch, Lockup, StakePool};
use solido_legacy_core::TOKENKEG_PROGRAM;

use crate::{pk_strat, u64_strat, u8_strat};

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SplStakePoolU64s<T> {
    pub total_lamports: T,
    pub pool_token_supply: T,
    pub last_update_epoch: T,
}

/// If `Option::None`, `any()` is used
#[derive(Debug, Clone, Default)]
pub struct GenStakePoolArgs {
    pub pool_mint: Option<BoxedStrategy<[u8; 32]>>,
    pub u64s: SplStakePoolU64s<Option<BoxedStrategy<u64>>>,
    pub stake_withdrawal_fee: Option<BoxedStrategy<Fee>>,
}

fn any_fee() -> impl Strategy<Value = Fee> {
    (0..=u64::MAX)
        .prop_flat_map(|d| (0..=d, Just(d)))
        .prop_map(|(numerator, denominator)| Fee {
            denominator,
            numerator,
        })
}

fn fee_strat(ovrride: Option<BoxedStrategy<Fee>>) -> BoxedStrategy<Fee> {
    ovrride.unwrap_or_else(|| any_fee().boxed())
}

fn any_future_epoch_fee() -> impl Strategy<Value = FutureEpoch<Fee>> {
    any_fee().prop_flat_map(|fee| {
        Union::new([
            Just(FutureEpoch::None),
            Just(FutureEpoch::One(fee)),
            Just(FutureEpoch::Two(fee)),
        ])
    })
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
struct SplStakePoolPksDontCare<T> {
    pub manager: T,
    pub staker: T,
    pub stake_deposit_authority: T,
    pub validator_list: T,
    pub reserve_stake: T,
    pub manager_fee_account: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
struct SplStakePoolU64sDontCare<T> {
    pub last_epoch_pool_token_supply: T,
    pub last_epoch_total_lamports: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
struct SplStakePoolU8sDontCare<T> {
    pub stake_withdraw_bump_seed: T,
    pub stake_referral_fee: T,
    pub sol_referral_fee: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
struct SplStakePoolFeesDontCare<T> {
    pub epoch_fee: T,
    pub sol_deposit_fee: T,
    pub sol_withdrawal_fee: T,
    pub stake_deposit_fee: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
struct SplStakePoolFutureFeesDontCare<T> {
    pub next_epoch_fee: T,
    pub next_stake_withdrawal_fee: T,
    pub next_sol_withdrawal_fee: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
struct SplStakePoolPkOptsDontCare<T> {
    pub sol_deposit_authority: T,
    pub sol_withdraw_authority: T,
    pub preferred_deposit_validator_vote_address: T,
    pub preferred_withdraw_validator_vote_address: T,
}

pub fn any_spl_stake_pool(
    GenStakePoolArgs {
        pool_mint,
        u64s,
        stake_withdrawal_fee,
    }: GenStakePoolArgs,
) -> impl Strategy<Value = StakePool> {
    (
        pk_strat(pool_mint),
        u64s.0.map(u64_strat),
        fee_strat(stake_withdrawal_fee),
        // dont cares, these fields should not affect INF program
        SplStakePoolPksDontCare::default().0.map(pk_strat),
        SplStakePoolU64sDontCare::default().0.map(u64_strat),
        SplStakePoolU8sDontCare::default().0.map(u8_strat),
        SplStakePoolFeesDontCare::default().0.map(fee_strat),
        core::array::from_fn::<_, SPL_STAKE_POOL_FUTURE_FEES_DONT_CARE_LEN, _>(|_| {
            any_future_epoch_fee()
        }),
        SplStakePoolPkOptsDontCare::default()
            .0
            .map(|none| proptest::option::of(pk_strat(none))),
    )
        .prop_map(
            |(
                pool_mint,
                u64s,
                stake_withdrawal_fee,
                pks_dc,
                u64s_dc,
                u8s_dc,
                fees_dc,
                fefs_dc,
                pk_opts_dc,
            )| {
                let u64s = SplStakePoolU64s(u64s);
                let pks_dc = SplStakePoolPksDontCare(pks_dc);
                let u64s_dc = SplStakePoolU64sDontCare(u64s_dc);
                let u8s_dc = SplStakePoolU8sDontCare(u8s_dc);
                let fees_dc = SplStakePoolFeesDontCare(fees_dc);
                let fefs_dc = SplStakePoolFutureFeesDontCare(fefs_dc);
                let pk_opts_dc = SplStakePoolPkOptsDontCare(pk_opts_dc);

                StakePool {
                    account_type: AccountType::StakePool,
                    total_lamports: *u64s.total_lamports(),
                    pool_token_supply: *u64s.pool_token_supply(),
                    last_update_epoch: *u64s.last_update_epoch(),
                    stake_withdrawal_fee,
                    pool_mint,
                    // assume tokenkeg
                    token_program_id: TOKENKEG_PROGRAM,
                    // assume public pool
                    lockup: Lockup::default(),
                    // dont cares, these fields should not affect INF program
                    manager: *pks_dc.manager(),
                    staker: *pks_dc.staker(),
                    stake_deposit_authority: *pks_dc.stake_deposit_authority(),
                    stake_withdraw_bump_seed: *u8s_dc.stake_withdraw_bump_seed(),
                    validator_list: *pks_dc.validator_list(),
                    reserve_stake: *pks_dc.reserve_stake(),
                    manager_fee_account: *pks_dc.manager_fee_account(),
                    epoch_fee: *fees_dc.epoch_fee(),
                    next_epoch_fee: *fefs_dc.next_epoch_fee(),
                    preferred_deposit_validator_vote_address: *pk_opts_dc
                        .preferred_deposit_validator_vote_address(),
                    preferred_withdraw_validator_vote_address: *pk_opts_dc
                        .preferred_withdraw_validator_vote_address(),
                    stake_deposit_fee: *fees_dc.stake_deposit_fee(),
                    next_stake_withdrawal_fee: *fefs_dc.next_stake_withdrawal_fee(),
                    stake_referral_fee: *u8s_dc.stake_referral_fee(),
                    sol_deposit_authority: *pk_opts_dc.sol_deposit_authority(),
                    sol_deposit_fee: *fees_dc.sol_deposit_fee(),
                    sol_referral_fee: *u8s_dc.sol_referral_fee(),
                    sol_withdraw_authority: *pk_opts_dc.sol_withdraw_authority(),
                    sol_withdrawal_fee: *fees_dc.sol_withdrawal_fee(),
                    next_sol_withdrawal_fee: *fefs_dc.next_sol_withdrawal_fee(),
                    last_epoch_pool_token_supply: *u64s_dc.last_epoch_pool_token_supply(),
                    last_epoch_total_lamports: *u64s_dc.last_epoch_total_lamports(),
                }
            },
        )
}

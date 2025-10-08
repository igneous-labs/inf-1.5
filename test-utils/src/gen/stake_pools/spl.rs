use generic_array_struct::generic_array_struct;
use proptest::{prelude::*, strategy::Union};
use sanctum_spl_stake_pool_core::{AccountType, Fee, FutureEpoch, StakePool};
use solido_legacy_core::TOKENKEG_PROGRAM;

use crate::{bool_strat, u64_strat};

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
    pub pool_mint: Option<BoxedStrategy<bool>>,
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

pub fn gen_spl_stake_pool(
    GenStakePoolArgs {
        pool_mint,
        u64s,
        stake_withdrawal_fee,
    }: GenStakePoolArgs,
) {
    todo!()
    // (
    //     bool_strat(pool_mint),
    //     u64s.0.map(u64_strat),
    //     fee_strat(stake_withdrawal_fee),
    // )
    //     .prop_map(|(pool_mint, u64s, stake_withdrawal_fee)| {})

    // StakePool {
    //     account_type: AccountType::StakePool,
    //     token_program_id: TOKENKEG_PROGRAM,
    //     total_lamports: *u64s.total_lamports(),
    //     pool_token_supply: *u64s.pool_token_supply(),
    //     last_update_epoch: *u64s.last_update_epoch(),
    //     stake_withdrawal_fee,
    //     pool_mint,
    //     // Other fields below do not affect INF program
    //     manager: any(),
    //     staker: any(),
    //     stake_deposit_authority: any(),
    //     stake_withdraw_bump_seed: any(),
    //     validator_list: any(),
    //     reserve_stake: any(),
    //     manager_fee_account: any(),
    //     lockup: any(),
    //     epoch_fee: any(),
    //     next_epoch_fee: any(),
    //     preferred_deposit_validator_vote_address: any(),
    //     preferred_withdraw_validator_vote_address: any(),
    //     stake_deposit_fee: any(),
    //     next_stake_withdrawal_fee: any(),
    //     stake_referral_fee: any(),
    //     sol_deposit_authority: any(),
    //     sol_deposit_fee: any(),
    //     sol_referral_fee: any(),
    //     sol_withdraw_authority: any(),
    //     sol_withdrawal_fee: any(),
    //     next_sol_withdrawal_fee: any(),
    //     last_epoch_pool_token_supply: any(),
    //     last_epoch_total_lamports: any(),
    // }
}

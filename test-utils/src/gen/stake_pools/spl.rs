use generic_array_struct::generic_array_struct;
use proptest::prelude::*;
use sanctum_spl_stake_pool_core::{Fee, StakePool};

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

pub fn any_spl_stake_pool(
    _pool_mint: [u8; 32],
    _u64s: SplStakePoolU64s<u64>,
    _stake_withdrawal_fee: Fee,
) -> StakePool {
    todo!()
    // StakePool {
    //     account_type: AccountType::StakePool,
    //     token_program_id: TOKENKEG_PROGRAM,
    //     total_lamports: *u64s.total_lamports(),
    //     pool_token_supply: *u64s.pool_token_supply(),
    //     last_update_epoch: *u64s.last_update_epoch(),
    //     manager: any(),
    //     staker: any(),
    //     stake_deposit_authority: any(),
    //     stake_withdraw_bump_seed: any(),
    //     validator_list: any(),
    //     reserve_stake: any(),
    //     pool_mint,
    //     manager_fee_account: any(),
    //     lockup: any(),
    //     epoch_fee: any(),
    //     next_epoch_fee: any(),
    //     preferred_deposit_validator_vote_address: any(),
    //     preferred_withdraw_validator_vote_address: any(),
    //     stake_deposit_fee: any(),
    //     stake_withdrawal_fee,
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

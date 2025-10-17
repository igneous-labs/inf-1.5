use std::ops::Range;

use inf1_core::instructions::liquidity::add::AddLiquidityIxAccs;
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    cpi::AddLiquidityPreAccountHandles,
    err::Inf1CtlErr,
    instructions::liquidity::{
        add::{AddLiquidityIxArgs, NewAddLiquidityIxPreAccsBuilder},
        IxPreAccs,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
};
use inf1_pp_jiminy::{
    cpi::price::lp::cpi_price_exact_in, instructions::price::exact_in::PriceExactInIxArgs,
};
use inf1_svc_jiminy::cpi::cpi_lst_to_sol;

use jiminy_cpi::{
    account::{AccountHandle, Accounts},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
    Cpi,
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::{
    RawTokenAccount, TokenAccount,
};

use crate::pricing_program::NewPPIxPreAccsBuilder;
use crate::svc::NewSvcIxPreAccsBuilder;
use crate::{
    instructions::sync_sol_value::sync_sol_val_with_retval,
    verify::{verify_not_input_disabled, verify_not_rebalancing_and_not_disabled, verify_pks},
};

pub type AddLiquidityIxAccounts<'acc> = AddLiquidityIxAccs<
    AccountHandle<'acc>,
    AddLiquidityPreAccountHandles<'acc>,
    Range<usize>,
    Range<usize>,
>;

/// Returns (prefix, sol_val_calc_program, remaining accounts, pricing_program, remaining accounts)

#[inline]
fn add_liquidity_accs_checked<'acc>(
    // TODO(pavs): Type error if max account not specified
    accounts: &Accounts<'acc, 64>,
    ix_args: AddLiquidityIxArgs,
) -> Result<AddLiquidityIxAccounts<'acc>, ProgramError> {
    if ix_args.amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    let (ix_prefix, suf) = accounts
        .as_slice()
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let ix_prefix = IxPreAccs(*ix_prefix);
    let list = LstStatePackedList::of_acc_data(accounts.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let lst_state = list
        .0
        .get(ix_args.lst_index as usize)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let lst_mint_acc = accounts.get(*ix_prefix.lst_mint());
    let token_prog = lst_mint_acc.owner();
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state() };

    let expected_reserves =
        create_raw_pool_reserves_addr(token_prog, &lst_state.mint, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let expected_pks = NewAddLiquidityIxPreAccsBuilder::start()
        .with_signer(accounts.get(*ix_prefix.signer()).key())
        .with_lst_mint(&lst_state.mint)
        .with_src_lst_acc(accounts.get(*ix_prefix.src_lst_acc()).key())
        .with_dst_lst_acc(accounts.get(*ix_prefix.dst_lst_acc()).key())
        .with_lp_token_mint(accounts.get(*ix_prefix.lp_token_mint()).key())
        .with_protocol_fee_accumulator(accounts.get(*ix_prefix.protocol_fee_accumulator()).key())
        .with_lst_token_program(accounts.get(*ix_prefix.lst_token_program()).key())
        .with_lp_token_program(accounts.get(*ix_prefix.lp_token_program()).key())
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_reserves(&expected_reserves)
        .build();

    verify_pks(accounts, &ix_prefix.0, &expected_pks.0)?;

    let calc_prog = suf.first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(accounts, &[*calc_prog], &[&lst_state.sol_value_calculator])?;

    // Taking out prog addres and calculating the number of accounts for lst_calc_program
    let calc_end = ix_prefix.0.len() + 1 + ix_args.lst_value_calc_accs as usize - 1;
    let pricing_start = calc_end + 1;

    // Get pricing program, first account after lst_calc_acc
    let pricing_prog = suf.get(pricing_start).ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let pricing_end = accounts.as_slice().len();

    verify_pks(accounts, &[*pricing_prog], &[&pool.pricing_program])?;

    verify_not_rebalancing_and_not_disabled(&pool)?;
    verify_not_input_disabled(&lst_state)?;

    Ok(AddLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog: *calc_prog,
        lst_calc: ix_prefix.0.len() + 1..accounts.as_slice().len(),
        pricing_prog: *pricing_prog,
        pricing: pricing_start..pricing_end,
    })
}

#[inline]
pub fn process_add_liquidity(
    accounts: &mut Accounts<'_, 64>,
    ix_args: AddLiquidityIxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    // Step 1: Verify pool is not rebalancing and not disabled
    // Step 2: Verify input not disabled for LST

    let AddLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog,
        lst_calc,
        pricing_prog,
        pricing,
    } = add_liquidity_accs_checked(accounts, ix_args)?;

    // Step 3: SyncSolValue for LST

    let lst_balance = RawTokenAccount::of_acc_data(accounts.get(*ix_prefix.pool_reserves()).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let retval = cpi_lst_to_sol(
        cpi,
        accounts,
        lst_calc_prog,
        lst_balance,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.lst_mint())
            .build(),
        &lst_calc,
    )?;

    sync_sol_val_with_retval(
        accounts,
        *ix_prefix.pool_state(),
        *ix_prefix.lst_state_list(),
        ix_args.lst_index as usize,
        retval,
    );

    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let start_total_sol_value = pool.total_sol_value;

    // Step 4: Calculate sol_value_to_add = LstToSol(amount).min

    let lst_amount_sol_value = cpi_lst_to_sol(
        cpi,
        accounts,
        lst_calc_prog,
        ix_args.amount,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.lst_mint())
            .build(),
        &lst_calc,
    )?;

    // Step 5: Calculate sol_value_to_add_after_fees = PriceLpTokensToMint(lp_tokens_sol_value)
    let lst_amount_sol_value_after_fees = cpi_price_exact_in(
        cpi,
        accounts,
        pricing_prog,
        PriceExactInIxArgs {
            sol_value: *lst_amount_sol_value.end(),
            amt: ix_args.amount,
        },
        NewPPIxPreAccsBuilder::start()
            .with_input_mint(*ix_prefix.lst_mint())
            .with_output_mint(*ix_prefix.lp_token_mint())
            .build(),
        pricing,
    )?;

    // Will dilute existing LPs if unchecked
    if lst_amount_sol_value_after_fees > *lst_amount_sol_value.end() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    Ok(())

    // Step 6: lp_fees_sol_value = lp_tokens_sol_value - sol_value_to_add_after_fees
    // Step 7: protocol_fees_sol_value = apply pool_state.lp_protocol_fee_bps to lp_fees_sol_value
    // Step 8: lp_tokens_due = sol_value_to_add_after_fees * lp_token_supply / pool_total_sol_value
    // Step 9: protocol_fees_lst = amount * protocol_fees_sol_value / sol_value_to_add
    // Step 10: Transfer protocol_fees_lst from src_lst_acc to protocol_fee_accumulator
    // Step 11: Transfer amount - protocol_fees_lst from src_lst_acc to pool_reserves
    // Step 12: Mint lp_tokens_due to dst_lp_token_acc
    // Step 13: SyncSolValue for LST
}

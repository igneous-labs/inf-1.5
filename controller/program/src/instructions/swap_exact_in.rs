use core::ops::Range;

use inf1_core::quote::swap::{exact_in::quote_exact_in, SwapQuoteArgs};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    cpi::{LstToSolRetVal, PricingRetVal, SolToLstRetVal},
    err::Inf1CtlErr,
    instructions::swap::{IxArgs, IxPreAccs},
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::U8Bool,
};
use inf1_pp_jiminy::{
    cpi::price::lp::cpi_price_exact_in, instructions::price::exact_in::PriceExactInIxArgs,
};
use inf1_svc_jiminy::cpi::{cpi_lst_to_sol, cpi_sol_to_lst};
use jiminy_cpi::{
    account::AccountHandle,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::{
    RawTokenAccount, TokenAccount,
};

use crate::{
    instructions::sync_sol_value::sync_sol_val_with_retval,
    pricing::NewPpIxPreAccsBuilder,
    svc::NewSvcIxPreAccsBuilder,
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks},
    Accounts, Cpi,
};

#[inline]
pub fn process_swap_exact_in(
    accounts: &mut Accounts<'_>,
    args: &IxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    if args.amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }
    let (ix_prefix, suf) = accounts
        .as_slice()
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let ix_prefix = IxPreAccs(*ix_prefix);

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
    verify_not_rebalancing_and_not_disabled(pool)?;

    let list = LstStatePackedList::of_acc_data(accounts.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    let inp_lst_state = list
        .0
        .get(args.inp_lst_index as usize)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let inp_lst_state = unsafe { inp_lst_state.as_lst_state() };

    let out_lst_state = list
        .0
        .get(args.out_lst_index as usize)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let out_lst_state = unsafe { out_lst_state.as_lst_state() };

    if U8Bool(&inp_lst_state.is_input_disabled).is_true() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled).into());
    }

    // Verify input calculator program
    let inp_calc_prog = *suf.first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(
        accounts,
        &[inp_calc_prog],
        &[&inp_lst_state.sol_value_calculator],
    )?;

    // Verify output calculator program
    let out_calc_prog = *suf
        .get(args.inp_lst_value_calc_accs as usize)
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(
        accounts,
        &[out_calc_prog],
        &[&out_lst_state.sol_value_calculator],
    )?;

    // Verify pricing program
    let pricing_prog = *suf
        .get(args.inp_lst_value_calc_accs as usize + args.out_lst_value_calc_accs as usize)
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(accounts, &[pricing_prog], &[&pool.pricing_program])?;

    // Define suf_ranges for the CPI calls
    let ix_prefix_len = ix_prefix.0.len();

    let inp_svc_accs_suf_range =
        ix_prefix_len + 1..ix_prefix_len + args.inp_lst_value_calc_accs as usize;

    let out_svc_accs_suf_range = ix_prefix_len + args.inp_lst_value_calc_accs as usize + 1
        ..ix_prefix_len
            + args.inp_lst_value_calc_accs as usize
            + args.out_lst_value_calc_accs as usize;

    let pricing_accs_suf_range = ix_prefix_len
        + args.inp_lst_value_calc_accs as usize
        + args.out_lst_value_calc_accs as usize
        + 1..accounts.as_slice().len();

    // Sync SOL values for LSTs
    lst_sync_sol_val(
        accounts,
        cpi,
        *ix_prefix.pool_state(),
        *ix_prefix.lst_state_list(),
        args.inp_lst_index as usize,
        *ix_prefix.inp_lst_mint(),
        *ix_prefix.inp_pool_reserves(),
        inp_calc_prog,
        inp_svc_accs_suf_range,
    )?;
    lst_sync_sol_val(
        accounts,
        cpi,
        *ix_prefix.pool_state(),
        *ix_prefix.lst_state_list(),
        args.out_lst_index as usize,
        *ix_prefix.out_lst_mint(),
        *ix_prefix.out_pool_reserves(),
        out_calc_prog,
        out_svc_accs_suf_range,
    )?;

    // Sync sol value for input LST
    let out_lst_balance =
        RawTokenAccount::of_acc_data(accounts.get(*ix_prefix.out_pool_reserves()).data())
            .and_then(TokenAccount::try_from_raw)
            .map(|a| a.amount())
            .ok_or(INVALID_ACCOUNT_DATA)?;

    let pool_trading_protocol_fee_bps = pool.trading_protocol_fee_bps;

    // TODO: Does `pool.total_sol_value` give the right value after doing sync?
    // Or should I do `accounts.get(...)` again, like in line 49?
    let start_total_sol_value = pool.total_sol_value;

    let inp_retval = cpi_lst_to_sol(
        cpi,
        accounts,
        inp_calc_prog,
        args.amount,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.inp_lst_mint())
            .build(),
        inp_svc_accs_suf_range,
    )?;

    let inp_sol_value = *inp_retval.start();
    if inp_sol_value == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    let out_sol_val = cpi_price_exact_in(
        cpi,
        accounts,
        pricing_prog,
        PriceExactInIxArgs {
            amt: args.amount,
            sol_value: *inp_retval.start(),
        },
        NewPpIxPreAccsBuilder::start()
            .with_input_mint(*ix_prefix.inp_lst_mint())
            .with_output_mint(*ix_prefix.out_lst_mint())
            .build(),
        pricing_accs_suf_range,
    )?;

    let out_retval = cpi_sol_to_lst(
        cpi,
        accounts,
        out_calc_prog,
        out_sol_val,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.out_lst_mint())
            .build(),
        out_svc_accs_suf_range,
    )?;

    let inp_calc = LstToSolRetVal(inp_retval);
    let out_calc = SolToLstRetVal(out_retval);
    let pricing = PricingRetVal(out_sol_val);

    let quote = quote_exact_in(SwapQuoteArgs {
        amt: args.amount,
        out_reserves: out_lst_balance,
        trading_protocol_fee_bps: pool_trading_protocol_fee_bps,
        inp_calc,
        out_calc,
        pricing,
        inp_mint: *accounts.get(*ix_prefix.inp_lst_mint()).key(),
        out_mint: *accounts.get(*ix_prefix.out_lst_mint()).key(),
    })
    .map_err(|_| Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;

    Ok(())
}

fn lst_sync_sol_val<'acc>(
    accounts: &mut Accounts<'acc>,
    cpi: &mut Cpi,
    pool_state: AccountHandle<'acc>,
    lst_state_list: AccountHandle<'acc>,
    lst_index: usize,
    lst_mint: AccountHandle<'acc>,
    lst_reserves: AccountHandle<'acc>,
    lst_calc_prog: AccountHandle<'acc>,
    suf_range: Range<usize>,
) -> Result<(), ProgramError> {
    // Sync sol value for input LST
    let lst_balance = RawTokenAccount::of_acc_data(accounts.get(lst_reserves).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let cpi_retval = cpi_lst_to_sol(
        cpi,
        accounts,
        lst_calc_prog,
        lst_balance,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(lst_mint)
            .build(),
        suf_range,
    )?;

    sync_sol_val_with_retval(accounts, pool_state, lst_state_list, lst_index, &cpi_retval)
}

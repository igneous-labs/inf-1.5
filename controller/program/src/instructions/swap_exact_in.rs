use inf1_core::{
    instructions::sync_sol_value::SyncSolValueIxAccs,
    quote::swap::{exact_in::quote_exact_in, SwapQuoteArgs},
};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    cpi::{LstToSolRetVal, PricingRetVal, SolToLstRetVal},
    err::Inf1CtlErr,
    instructions::{
        swap::{exact_in::NewSwapExactInIxPreAccsBuilder, IxArgs, IxPreAccs},
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_BUMP, POOL_STATE_ID},
    pda_onchain::{create_raw_pool_reserves_addr, create_raw_protocol_fee_accumulator_addr},
    program_err::Inf1CtlCustomProgErr,
    typedefs::{
        lst_state::{LstState, LstStatePacked},
        u8bool::U8Bool,
    },
};
use inf1_pp_jiminy::{
    cpi::price::lp::cpi_price_exact_in, instructions::price::exact_in::PriceExactInIxArgs,
};
use inf1_svc_jiminy::cpi::{cpi_lst_to_sol, cpi_sol_to_lst};
use jiminy_cpi::{
    account::AccountHandle,
    pda::{PdaSeed, PdaSigner},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use sanctum_spl_token_jiminy::{
    instructions::transfer::transfer_checked_ix_account_handle_perms,
    sanctum_spl_token_core::{
        instructions::transfer::{NewTransferCheckedIxAccsBuilder, TransferCheckedIxData},
        state::{
            account::{RawTokenAccount, TokenAccount},
            mint::{Mint, RawMint},
        },
    },
};

use crate::{
    pricing::NewPpIxPreAccsBuilder,
    svc::{lst_sync_sol_val_unchecked, NewSvcIxPreAccsBuilder},
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

    let pool_state = *ix_prefix.pool_state();
    let lst_state_list = *ix_prefix.lst_state_list();
    let inp_lst_mint = *ix_prefix.inp_lst_mint();
    let out_lst_mint = *ix_prefix.out_lst_mint();
    let inp_pool_reserves = *ix_prefix.inp_pool_reserves();
    let out_pool_reserves = *ix_prefix.out_pool_reserves();

    let list = LstStatePackedList::of_acc_data(accounts.get(lst_state_list).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    let (inp_lst_state, inp_token_prog, expected_inp_reserves) =
        get_lst_state(accounts, &list, args.inp_lst_index as usize, inp_lst_mint)?;
    let (out_lst_state, out_token_prog, expected_out_reserves) =
        get_lst_state(accounts, &list, args.out_lst_index as usize, out_lst_mint)?;

    let expected_protocol_fee_accumulator = create_raw_protocol_fee_accumulator_addr(
        out_token_prog,
        &out_lst_state.mint,
        &out_lst_state.protocol_fee_accumulator_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(
        Inf1CtlErr::InvalidProtocolFeeAccumulator,
    ))?;

    // Verify incoming accounts
    let expected_pks = NewSwapExactInIxPreAccsBuilder::start()
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_state(&POOL_STATE_ID)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_inp_pool_reserves(&expected_inp_reserves)
        .with_out_pool_reserves(&expected_out_reserves)
        .with_inp_lst_mint(&inp_lst_state.mint)
        .with_out_lst_mint(&out_lst_state.mint)
        .with_inp_lst_token_program(inp_token_prog)
        .with_out_lst_token_program(out_token_prog)
        // NOTE: For the following accounts, it's okay to use the same ones passed by the user since the CPIs would fail if they're not as expected.
        // User can't pass the `inp_lst_reserves` as `inp_lst_acc` because we're also not doing `invoke_signed` for the `inp_lst` transfer.
        .with_inp_lst_acc(accounts.get(*ix_prefix.inp_lst_acc()).key())
        .with_out_lst_acc(accounts.get(*ix_prefix.out_lst_acc()).key())
        .with_signer(accounts.get(*ix_prefix.signer()).key())
        .build();

    verify_pks(accounts, &ix_prefix.0, &expected_pks.0)?;

    if U8Bool(&inp_lst_state.is_input_disabled).is_true() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled).into());
    }

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(pool_state).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
    verify_not_rebalancing_and_not_disabled(pool)?;

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

    let out_svc_accs_suf_range = inp_svc_accs_suf_range.end + 1
        ..inp_svc_accs_suf_range.end + args.out_lst_value_calc_accs as usize;

    let pricing_accs_suf_range = out_svc_accs_suf_range.end + 1..accounts.as_slice().len();

    // Sync SOL values for LSTs
    lst_sync_sol_val_unchecked(
        accounts,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.inp_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.inp_pool_reserves())
                .build(),
            calc_prog: inp_calc_prog,
            calc: inp_svc_accs_suf_range.clone(),
        },
        args.inp_lst_index as usize,
    )?;
    lst_sync_sol_val_unchecked(
        accounts,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.out_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.out_pool_reserves())
                .build(),
            calc_prog: out_calc_prog,
            calc: out_svc_accs_suf_range.clone(),
        },
        args.out_lst_index as usize,
    )?;

    let out_lst_balance = RawTokenAccount::of_acc_data(accounts.get(out_pool_reserves).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let pool = unsafe { PoolState::of_acc_data(accounts.get(pool_state).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let pool_trading_protocol_fee_bps = pool.trading_protocol_fee_bps;

    let start_total_sol_value = pool.total_sol_value;

    let inp_retval = cpi_lst_to_sol(
        cpi,
        accounts,
        inp_calc_prog,
        args.amount,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.inp_lst_mint())
            .build(),
        inp_svc_accs_suf_range.clone(),
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
        out_svc_accs_suf_range.clone(),
    )?;

    if *out_retval.start() < args.limit {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

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

    let inp_lst_token_program = *accounts.get(*ix_prefix.inp_lst_token_program()).key();
    let inp_lst_decimals = RawMint::of_acc_data(accounts.get(*ix_prefix.inp_lst_mint()).data())
        .and_then(Mint::try_from_raw)
        .map(|a| a.decimals())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let inp_lst_transfer_accs = NewTransferCheckedIxAccsBuilder::start()
        .with_auth(*ix_prefix.signer())
        .with_mint(*ix_prefix.inp_lst_mint())
        .with_src(*ix_prefix.inp_lst_acc())
        .with_dst(inp_pool_reserves)
        .build();

    cpi.invoke_fwd(
        accounts,
        &inp_lst_token_program,
        TransferCheckedIxData::new(args.amount, inp_lst_decimals).as_buf(),
        inp_lst_transfer_accs.0,
    )?;

    let out_lst_token_program = *accounts.get(*ix_prefix.out_lst_token_program()).key();
    let out_lst_decimals = RawMint::of_acc_data(accounts.get(*ix_prefix.out_lst_mint()).data())
        .and_then(Mint::try_from_raw)
        .map(|a| a.decimals())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let protocol_fee_transfer_accs = transfer_checked_ix_account_handle_perms(
        NewTransferCheckedIxAccsBuilder::start()
            .with_auth(*ix_prefix.pool_state())
            .with_mint(*ix_prefix.out_lst_mint())
            .with_src(out_pool_reserves)
            .with_dst(*ix_prefix.protocol_fee_accumulator())
            .build(),
    );

    let signers_seeds = &[
        PdaSeed::new(POOL_STATE_ID.as_slice()),
        PdaSeed::new(&[POOL_STATE_BUMP]),
    ];

    cpi.invoke_signed(
        accounts,
        &out_lst_token_program,
        TransferCheckedIxData::new(quote.0.protocol_fee, out_lst_decimals).as_buf(),
        protocol_fee_transfer_accs,
        &[PdaSigner::new(signers_seeds)],
    )?;

    let out_lst_transfer_accs = transfer_checked_ix_account_handle_perms(
        NewTransferCheckedIxAccsBuilder::start()
            .with_auth(*ix_prefix.pool_state())
            .with_mint(*ix_prefix.out_lst_mint())
            .with_src(out_pool_reserves)
            .with_dst(*ix_prefix.out_lst_acc())
            .build(),
    );

    cpi.invoke_signed(
        accounts,
        &out_lst_token_program,
        TransferCheckedIxData::new(quote.0.out, out_lst_decimals).as_buf(),
        out_lst_transfer_accs,
        &[PdaSigner::new(signers_seeds)],
    )?;

    // Sync SOL values for LSTs
    lst_sync_sol_val_unchecked(
        accounts,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.inp_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.inp_pool_reserves())
                .build(),
            calc_prog: inp_calc_prog,
            calc: inp_svc_accs_suf_range,
        },
        args.inp_lst_index as usize,
    )?;
    lst_sync_sol_val_unchecked(
        accounts,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.out_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.out_pool_reserves())
                .build(),
            calc_prog: out_calc_prog,
            calc: out_svc_accs_suf_range,
        },
        args.out_lst_index as usize,
    )?;

    let pool = unsafe { PoolState::of_acc_data(accounts.get(pool_state).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let final_total_sol_value = pool.total_sol_value;

    if final_total_sol_value < start_total_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    Ok(())
}

fn get_lst_state<'a, 'b>(
    accounts: &'a Accounts<'b>,
    list: &'a LstStatePackedList,
    idx: usize,
    lst_mint: AccountHandle<'a>,
) -> Result<(&'a LstState, &'a [u8; 32], [u8; 32]), ProgramError> {
    let lst_state: &LstStatePacked = list
        .0
        .get(idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let lst_state = unsafe { lst_state.as_lst_state() };

    let token_prog = accounts.get(lst_mint).key();
    let expected_reserves =
        create_raw_pool_reserves_addr(token_prog, &lst_state.mint, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    Ok((lst_state, token_prog, expected_reserves))
}

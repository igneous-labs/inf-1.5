use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList, pool_state::PoolState,
        rebalance_record::RebalanceRecord,
    },
    cpi::StartRebalanceIxPreAccountHandles,
    err::Inf1CtlErr,
    instructions::{
        rebalance::{
            end::{END_REBALANCE_IX_DISCM, END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT},
            start::{
                NewStartRebalanceIxPreAccsBuilder, StartRebalanceIxArgs, StartRebalanceIxPreAccs,
                START_REBALANCE_IX_PRE_IS_SIGNER,
            },
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{
        INSTRUCTIONS_SYSVAR_ID, LST_STATE_LIST_ID, POOL_STATE_BUMP, POOL_STATE_ID,
        REBALANCE_RECORD_ID,
    },
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::U8BoolMut,
    ID,
};
use jiminy_cpi::{
    account::{Abr, Account, AccountHandle},
    pda::{PdaSeed, PdaSigner},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_instructions::Instructions;

use inf1_core::instructions::{
    rebalance::start::StartRebalanceIxAccs, sync_sol_value::SyncSolValueIxAccs,
};

use sanctum_spl_token_jiminy::{
    instructions::transfer::transfer_checked_ix_account_handle_perms,
    sanctum_spl_token_core::{
        instructions::transfer::{NewTransferCheckedIxAccsBuilder, TransferCheckedIxData},
        state::mint::{Mint, RawMint},
    },
};

use core::mem::size_of;

use sanctum_system_jiminy::{
    instructions::assign::assign_ix_account_handle_perms,
    sanctum_system_core::{
        instructions::assign::{AssignIxData, NewAssignIxAccsBuilder},
        ID as SYSTEM_PROGRAM_ID,
    },
};

use crate::{
    svc::lst_sync_sol_val_unchecked,
    token::get_token_account_amount,
    verify::{
        log_and_return_acc_privilege_err, verify_not_rebalancing_and_not_disabled, verify_pks,
        verify_signers,
    },
    Cpi,
};

pub type StartRebalanceIxAccounts<'a, 'acc> = StartRebalanceIxAccs<
    AccountHandle<'acc>,
    StartRebalanceIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
>;

/// Verify that an EndRebalance instruction exists after the current instruction with the expected destination mint
#[inline]
fn verify_end_rebalance_exists(
    instructions_acc: &Account,
    expected_inp_lst_mint: &[u8; 32],
) -> Result<(), ProgramError> {
    let instructions =
        Instructions::try_from_account(instructions_acc).ok_or(INVALID_ACCOUNT_DATA)?;

    let next_end_rebalance = instructions
        .iter()
        .skip(instructions.current_idx() + 1)
        .find(|intro_instr| {
            intro_instr.program_id() == &ID
                && intro_instr.data().first().copied() == Some(END_REBALANCE_IX_DISCM)
        })
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::NoSucceedingEndRebalance))?;

    let inp_lst_mint = next_end_rebalance
        .accounts()
        .get(END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::NoSucceedingEndRebalance))?
        .key();

    if inp_lst_mint != expected_inp_lst_mint {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::NoSucceedingEndRebalance).into());
    }

    Ok(())
}

fn start_rebalance_accs_checked<'a, 'acc>(
    abr: &Abr,
    accounts: &'a [AccountHandle<'acc>],
    args: &StartRebalanceIxArgs,
) -> Result<StartRebalanceIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = StartRebalanceIxPreAccs(*ix_prefix);

    let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
    let list = LstStatePackedList::of_acc_data(abr.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    let out_lst_idx = args.out_lst_index as usize;
    let out_lst_state = list
        .0
        .get(out_lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let inp_lst_idx = args.inp_lst_index as usize;
    let inp_lst_state = list
        .0
        .get(inp_lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let out_lst_state = unsafe { out_lst_state.as_lst_state() };
    let inp_lst_state = unsafe { inp_lst_state.as_lst_state() };

    if inp_lst_state.is_input_disabled != 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled).into());
    }

    let instructions_acc = abr.get(*ix_prefix.instructions());

    verify_end_rebalance_exists(instructions_acc, abr.get(*ix_prefix.inp_lst_mint()).key())?;

    let out_lst_mint_acc = abr.get(*ix_prefix.out_lst_mint());
    let out_token_prog = out_lst_mint_acc.owner();

    let inp_lst_mint_acc = abr.get(*ix_prefix.inp_lst_mint());
    let inp_token_prog = inp_lst_mint_acc.owner();

    let expected_out_reserves = create_raw_pool_reserves_addr(
        out_token_prog,
        &out_lst_state.mint,
        &out_lst_state.pool_reserves_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let expected_inp_reserves = create_raw_pool_reserves_addr(
        inp_token_prog,
        &inp_lst_state.mint,
        &inp_lst_state.pool_reserves_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let expected_pks = NewStartRebalanceIxPreAccsBuilder::start()
        .with_rebalance_auth(&pool.rebalance_authority)
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_rebalance_record(&REBALANCE_RECORD_ID)
        .with_out_lst_mint(&out_lst_state.mint)
        .with_inp_lst_mint(&inp_lst_state.mint)
        .with_out_pool_reserves(&expected_out_reserves)
        .with_inp_pool_reserves(&expected_inp_reserves)
        .with_withdraw_to(abr.get(*ix_prefix.withdraw_to()).key())
        .with_instructions(&INSTRUCTIONS_SYSVAR_ID)
        .with_system_program(&SYSTEM_PROGRAM_ID)
        .with_out_lst_token_program(out_token_prog)
        .build();
    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    verify_signers(abr, &ix_prefix.0, &START_REBALANCE_IX_PRE_IS_SIGNER.0)
        .map_err(|expected_signer| log_and_return_acc_privilege_err(abr, *expected_signer))?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let out_calc_accs_len = args.out_lst_value_calc_accs as usize;
    if out_calc_accs_len == 0 {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    }
    if suf.len() < out_calc_accs_len + 1 {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    }

    let (out_calc_prog, out_suf) = suf.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let (out_calc, inp_suf) = out_suf.split_at(out_calc_accs_len - 1);

    let (inp_calc_prog, inp_calc) = inp_suf.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    verify_pks(
        abr,
        &[*out_calc_prog, *inp_calc_prog],
        &[
            &out_lst_state.sol_value_calculator,
            &inp_lst_state.sol_value_calculator,
        ],
    )?;

    let out_reserves_balance =
        get_token_account_amount(abr.get(*ix_prefix.out_pool_reserves()).data())?;
    if out_reserves_balance < args.min_starting_out_lst {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let inp_reserves_balance =
        get_token_account_amount(abr.get(*ix_prefix.inp_pool_reserves()).data())?;
    if inp_reserves_balance > args.max_starting_inp_lst {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    Ok(StartRebalanceIxAccounts {
        ix_prefix,
        out_calc_prog: *out_calc_prog,
        out_calc,
        inp_calc_prog: *inp_calc_prog,
        inp_calc,
    })
}

#[inline]
pub fn process_start_rebalance(
    abr: &mut Abr,
    accounts: &[AccountHandle],
    args: StartRebalanceIxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let StartRebalanceIxAccounts {
        ix_prefix,
        out_calc_prog,
        out_calc,
        inp_calc_prog,
        inp_calc,
    } = start_rebalance_accs_checked(abr, accounts, &args)?;

    let out_lst_idx = args.out_lst_index as usize;
    let inp_lst_idx = args.inp_lst_index as usize;

    lst_sync_sol_val_unchecked(
        abr,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.out_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.out_pool_reserves())
                .build(),
            calc_prog: out_calc_prog,
            calc: out_calc,
        },
        out_lst_idx,
    )?;

    lst_sync_sol_val_unchecked(
        abr,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.inp_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.inp_pool_reserves())
                .build(),
            calc_prog: inp_calc_prog,
            calc: inp_calc,
        },
        inp_lst_idx,
    )?;

    let old_total_sol_value = {
        let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
        pool.total_sol_value
    };

    // Transfer out_lst tokens from reserves to withdraw_to account
    let out_lst_mint_data = abr.get(*ix_prefix.out_lst_mint()).data();
    let out_lst_mint = RawMint::of_acc_data(out_lst_mint_data)
        .and_then(Mint::try_from_raw)
        .ok_or(INVALID_ACCOUNT_DATA)?;
    let decimals = out_lst_mint.decimals();

    let transfer_checked_ix_data = TransferCheckedIxData::new(args.amount, decimals);
    let transfer_checked_accs = NewTransferCheckedIxAccsBuilder::start()
        .with_src(*ix_prefix.out_pool_reserves())
        .with_mint(*ix_prefix.out_lst_mint())
        .with_dst(*ix_prefix.withdraw_to())
        .with_auth(*ix_prefix.pool_state())
        .build();
    let out_lst_token_program_key = *abr.get(*ix_prefix.out_lst_token_program()).key();

    cpi.invoke_signed(
        abr,
        &out_lst_token_program_key,
        transfer_checked_ix_data.as_buf(),
        transfer_checked_ix_account_handle_perms(transfer_checked_accs),
        &[PdaSigner::new(&[
            PdaSeed::new(b"state"),
            PdaSeed::new(&[POOL_STATE_BUMP]),
        ])],
    )?;

    lst_sync_sol_val_unchecked(
        abr,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.out_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.out_pool_reserves())
                .build(),
            calc_prog: out_calc_prog,
            calc: out_calc,
        },
        out_lst_idx,
    )?;

    let rebalance_record_seeds = [
        PdaSeed::new(b"rebalance-record"),
        PdaSeed::new(&[inf1_ctl_jiminy::keys::REBALANCE_RECORD_BUMP]),
    ];
    let rebalance_record_signer = PdaSigner::new(&rebalance_record_seeds);

    cpi.invoke_signed(
        abr,
        &SYSTEM_PROGRAM_ID,
        AssignIxData::new(&ID).as_buf(),
        assign_ix_account_handle_perms(
            NewAssignIxAccsBuilder::start()
                .with_assign(*ix_prefix.rebalance_record())
                .build(),
        ),
        &[rebalance_record_signer],
    )?;

    abr.transfer_direct(*ix_prefix.pool_state(), *ix_prefix.rebalance_record(), 1)?;

    let rebalance_record_space = size_of::<RebalanceRecord>();
    abr.get_mut(*ix_prefix.rebalance_record())
        .realloc(rebalance_record_space, false)?;

    let rebalance_record_acc = abr.get_mut(*ix_prefix.rebalance_record());
    let rr = unsafe { RebalanceRecord::of_acc_data_mut(rebalance_record_acc.data_mut()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidRebalanceRecordData))?;

    rr.dst_lst_index = args.inp_lst_index;
    rr.old_total_sol_value = old_total_sol_value;

    let pool_acc = abr.get_mut(*ix_prefix.pool_state());
    let pool = unsafe { PoolState::of_acc_data_mut(pool_acc.data_mut()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
    U8BoolMut(&mut pool.is_rebalancing).set_true();

    Ok(())
}

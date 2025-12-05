use core::mem::size_of;

use inf1_core::instructions::rebalance::start::StartRebalanceIxAccs;
use inf1_ctl_jiminy::{
    account_utils::{
        lst_state_list_checked, pool_state_v2_checked, pool_state_v2_checked_mut,
        rebalance_record_checked_mut,
    },
    accounts::rebalance_record::RebalanceRecord,
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
    keys::{INSTRUCTIONS_SYSVAR_ID, LST_STATE_LIST_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
    pda_onchain::{create_raw_pool_reserves_addr, POOL_STATE_SIGNER, REBALANCE_RECORD_SIGNER},
    program_err::Inf1CtlCustomProgErr,
    sync_sol_val::SyncSolVal,
    typedefs::u8bool::U8BoolMut,
    ID,
};
use jiminy_cpi::{
    account::{Abr, Account, AccountHandle},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use jiminy_sysvar_clock::Clock;
use jiminy_sysvar_instructions::Instructions;
use sanctum_spl_token_jiminy::{
    instructions::transfer::transfer_checked_ix_account_handle_perms,
    sanctum_spl_token_core::instructions::transfer::{
        NewTransferCheckedIxAccsBuilder, TransferCheckedIxData,
    },
};
use sanctum_system_jiminy::{
    instructions::assign::assign_ix_account_handle_perms,
    sanctum_system_core::{
        instructions::assign::{AssignIxData, NewAssignIxAccsBuilder},
        ID as SYSTEM_PROGRAM_ID,
    },
};

use crate::{
    svc::{cpi_lst_reserves_sol_val, lst_ssv_uy, update_lst_state_sol_val, SyncSolValIxAccounts},
    token::{checked_mint_of, get_token_account_amount},
    utils::{accs_split_first_chunk, split_suf_accs},
    verify::{
        verify_not_input_disabled, verify_not_rebalancing_and_not_disabled, verify_pks,
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
                && intro_instr.data().first() == Some(&END_REBALANCE_IX_DISCM)
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
    StartRebalanceIxArgs {
        out_lst_value_calc_accs,
        out_lst_index,
        inp_lst_index,
        min_starting_out_lst,
        max_starting_inp_lst,
        amount: _,
    }: &StartRebalanceIxArgs,
) -> Result<StartRebalanceIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accs_split_first_chunk(accounts)?;
    let ix_prefix = StartRebalanceIxPreAccs(*ix_prefix);

    let pool = pool_state_v2_checked(abr.get(*ix_prefix.pool_state()))?;
    let list = lst_state_list_checked(abr.get(*ix_prefix.lst_state_list()))?;

    let [i, o] = [
        (inp_lst_index, ix_prefix.inp_lst_mint()),
        (out_lst_index, ix_prefix.out_lst_mint()),
    ]
    .map(|(i, mint_handle)| {
        let lst_state = list.0.get(*i as usize).ok_or(Inf1CtlErr::InvalidLstIndex)?;
        let token_prog = abr.get(*mint_handle).owner();
        let reserves = create_raw_pool_reserves_addr(
            token_prog,
            &lst_state.mint,
            &lst_state.pool_reserves_bump,
        )
        .ok_or(Inf1CtlErr::InvalidReserves)?;
        Ok::<_, Inf1CtlCustomProgErr>((lst_state, token_prog, reserves))
    });
    let (inp_lst_state, _inp_token_prog, expected_inp_reserves) = i?;
    let (out_lst_state, out_token_prog, expected_out_reserves) = o?;

    verify_not_input_disabled(inp_lst_state)?;

    let expected_pks = NewStartRebalanceIxPreAccsBuilder::start()
        .with_rebalance_auth(&pool.rebalance_authority)
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_rebalance_record(&REBALANCE_RECORD_ID)
        .with_out_lst_mint(&out_lst_state.mint)
        .with_inp_lst_mint(&inp_lst_state.mint)
        .with_out_pool_reserves(&expected_out_reserves)
        .with_inp_pool_reserves(&expected_inp_reserves)
        .with_instructions(&INSTRUCTIONS_SYSVAR_ID)
        .with_system_program(&SYSTEM_PROGRAM_ID)
        .with_out_lst_token_program(out_token_prog)
        // Free account - caller can specify any destination for withdrawn tokens
        .with_withdraw_to(abr.get(*ix_prefix.withdraw_to()).key())
        .build();
    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    verify_signers(abr, &ix_prefix.0, &START_REBALANCE_IX_PRE_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let [(out_calc_prog, out_calc), (inp_calc_prog, inp_calc)] =
        split_suf_accs(suf, &[*out_lst_value_calc_accs])?;

    verify_pks(
        abr,
        &[out_calc_prog, inp_calc_prog],
        &[
            &out_lst_state.sol_value_calculator,
            &inp_lst_state.sol_value_calculator,
        ],
    )?;

    let out_reserves_balance = get_token_account_amount(abr.get(*ix_prefix.out_pool_reserves()))?;
    if out_reserves_balance < *min_starting_out_lst {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let inp_reserves_balance = get_token_account_amount(abr.get(*ix_prefix.inp_pool_reserves()))?;
    if inp_reserves_balance > *max_starting_inp_lst {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let instructions_acc = abr.get(*ix_prefix.instructions());
    verify_end_rebalance_exists(instructions_acc, &inp_lst_state.mint)?;

    // allow start lst = end lst
    // with no additional special case handling
    // (e.g. not calling SyncSolVal twice)

    Ok(StartRebalanceIxAccounts {
        ix_prefix,
        out_calc_prog,
        out_calc,
        inp_calc_prog,
        inp_calc,
    })
}

#[inline]
pub fn process_start_rebalance(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accounts: &[AccountHandle],
    args: &StartRebalanceIxArgs,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let StartRebalanceIxAccounts {
        ix_prefix,
        out_calc_prog,
        out_calc,
        inp_calc_prog,
        inp_calc,
    } = start_rebalance_accs_checked(abr, accounts, args)?;

    pool_state_v2_checked_mut(abr.get_mut(*ix_prefix.pool_state()))?
        .release_yield(clock.slot)
        .map_err(Inf1CtlCustomProgErr)?;

    // TODO: see if we can factor this common code out with
    // `sync_pair_accs` in swap

    let [inp_lst_index, out_lst_index] =
        [args.inp_lst_index, args.out_lst_index].map(|x| x as usize);
    let [inp_accs, out_accs] = [
        (
            ix_prefix.inp_lst_mint(),
            ix_prefix.inp_pool_reserves(),
            inp_calc_prog,
            inp_calc,
        ),
        (
            ix_prefix.out_lst_mint(),
            ix_prefix.out_pool_reserves(),
            out_calc_prog,
            out_calc,
        ),
    ]
    .map(|(mint, reserves, calc_prog, calc)| SyncSolValIxAccounts {
        ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
            .with_pool_state(*ix_prefix.pool_state())
            .with_lst_state_list(*ix_prefix.lst_state_list())
            .with_lst_mint(*mint)
            .with_pool_reserves(*reserves)
            .build(),
        calc_prog: *abr.get(calc_prog).key(),
        calc,
    });

    [(inp_accs, inp_lst_index), (out_accs, out_lst_index)]
        .iter()
        .try_for_each(|(accs, idx)| lst_ssv_uy(abr, cpi, accs, *idx))?;

    let old_total_sol_value = {
        let pool = pool_state_v2_checked(abr.get(*ix_prefix.pool_state()))?;
        pool.total_sol_value
    };

    // Transfer out_lst tokens from reserves to withdraw_to account.
    let decimals = checked_mint_of(abr.get(*ix_prefix.out_lst_mint()))?.decimals();
    let transfer_checked_ix_data = TransferCheckedIxData::new(args.amount, decimals);
    let transfer_checked_accs = NewTransferCheckedIxAccsBuilder::start()
        .with_src(*ix_prefix.out_pool_reserves())
        .with_mint(*ix_prefix.out_lst_mint())
        .with_dst(*ix_prefix.withdraw_to())
        .with_auth(*ix_prefix.pool_state())
        .build();
    cpi.invoke_signed_handle(
        abr,
        *ix_prefix.out_lst_token_program(),
        transfer_checked_ix_data.as_buf(),
        transfer_checked_ix_account_handle_perms(transfer_checked_accs),
        &[POOL_STATE_SIGNER],
    )?;

    // sync sol val with new decreased out_pool_reserves balance,
    // but dont update_yield
    let out_lst_new = cpi_lst_reserves_sol_val(abr, cpi, &out_accs)?;
    let out_lst_sol_val = update_lst_state_sol_val(
        abr,
        *out_accs.ix_prefix.lst_state_list(),
        out_lst_index,
        out_lst_new,
    )?;
    let ps = pool_state_v2_checked_mut(abr.get_mut(*ix_prefix.pool_state()))?;
    let new_total = SyncSolVal {
        lst_sol_val: out_lst_sol_val,
    }
    .exec(ps.total_sol_value)
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;
    ps.total_sol_value = new_total;

    U8BoolMut(&mut ps.is_rebalancing).set_true();

    // setup RebalanceRecord

    cpi.invoke_signed(
        abr,
        &SYSTEM_PROGRAM_ID,
        AssignIxData::new(&ID).as_buf(),
        assign_ix_account_handle_perms(
            NewAssignIxAccsBuilder::start()
                .with_assign(*ix_prefix.rebalance_record())
                .build(),
        ),
        &[REBALANCE_RECORD_SIGNER],
    )?;

    // hot potato
    abr.transfer_direct(*ix_prefix.pool_state(), *ix_prefix.rebalance_record(), 1)?;

    let rebalance_record_space = size_of::<RebalanceRecord>();
    abr.get_mut(*ix_prefix.rebalance_record())
        .realloc(rebalance_record_space, false)?;

    let rr = rebalance_record_checked_mut(abr.get_mut(*ix_prefix.rebalance_record()))?;
    rr.inp_lst_index = args.inp_lst_index;
    rr.old_total_sol_value = old_total_sol_value;

    Ok(())
}

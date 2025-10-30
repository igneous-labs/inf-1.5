use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList, pool_state::PoolState,
        rebalance_record::RebalanceRecord,
    },
    cpi::EndRebalanceIxPreAccountHandles,
    err::Inf1CtlErr,
    instructions::{
        rebalance::end::{
            EndRebalanceIxPreAccs, NewEndRebalanceIxPreAccsBuilder, END_REBALANCE_IX_PRE_IS_SIGNER,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::U8BoolMut,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    pda::{PdaSeed, PdaSigner},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};

use inf1_core::instructions::{
    rebalance::end::EndRebalanceIxAccs, sync_sol_value::SyncSolValueIxAccs,
};

use sanctum_system_jiminy::{
    instructions::assign::assign_ix_account_handle_perms,
    sanctum_system_core::{
        instructions::assign::{AssignIxData, NewAssignIxAccsBuilder},
        ID as SYSTEM_PROGRAM_ID,
    },
};

use crate::{
    svc::lst_sync_sol_val_unchecked,
    verify::{
        log_and_return_acc_privilege_err, verify_is_rebalancing, verify_pks, verify_signers,
        verify_sol_value_calculator_is_program,
    },
    Cpi,
};

pub type EndRebalanceIxAccounts<'a, 'acc> = EndRebalanceIxAccs<
    AccountHandle<'acc>,
    EndRebalanceIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
>;

fn end_rebalance_accs_checked<'a, 'acc>(
    abr: &Abr,
    accounts: &'a [AccountHandle<'acc>],
) -> Result<EndRebalanceIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = EndRebalanceIxPreAccs(*ix_prefix);

    let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
    let list = LstStatePackedList::of_acc_data(abr.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    verify_is_rebalancing(pool)?;

    let rebalance_record_acc = abr.get(*ix_prefix.rebalance_record());
    let rr = unsafe { RebalanceRecord::of_acc_data(rebalance_record_acc.data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidRebalanceRecordData))?;

    let inp_lst_idx = rr.dst_lst_index as usize;
    let inp_lst_state = list
        .0
        .get(inp_lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    let inp_lst_state = unsafe { inp_lst_state.as_lst_state() };

    let inp_lst_mint_acc = abr.get(*ix_prefix.inp_lst_mint());
    if inp_lst_mint_acc.key() != &inp_lst_state.mint {
        return Err(INVALID_ACCOUNT_DATA.into());
    }

    let inp_token_prog = inp_lst_mint_acc.owner();
    let expected_inp_reserves = create_raw_pool_reserves_addr(
        inp_token_prog,
        &inp_lst_state.mint,
        &inp_lst_state.pool_reserves_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let expected_pks = NewEndRebalanceIxPreAccsBuilder::start()
        .with_rebalance_auth(&pool.rebalance_authority)
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_rebalance_record(&REBALANCE_RECORD_ID)
        .with_inp_lst_mint(&inp_lst_state.mint)
        .with_inp_pool_reserves(&expected_inp_reserves)
        .with_system_program(&SYSTEM_PROGRAM_ID)
        .build();
    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    verify_signers(abr, &ix_prefix.0, &END_REBALANCE_IX_PRE_IS_SIGNER.0)
        .map_err(|expected_signer| log_and_return_acc_privilege_err(abr, *expected_signer))?;

    let (inp_calc_prog, inp_calc) = suf.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_sol_value_calculator_is_program(abr.get(*inp_calc_prog))?;

    verify_pks(
        abr,
        &[*inp_calc_prog],
        &[&inp_lst_state.sol_value_calculator],
    )?;

    Ok(EndRebalanceIxAccounts {
        ix_prefix,
        inp_calc_prog: *inp_calc_prog,
        inp_calc,
    })
}

#[inline]
pub fn process_end_rebalance(
    abr: &mut Abr,
    accounts: &[AccountHandle],
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let EndRebalanceIxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
    } = end_rebalance_accs_checked(abr, accounts)?;

    {
        let pool_acc = abr.get_mut(*ix_prefix.pool_state());
        let pool = unsafe { PoolState::of_acc_data_mut(pool_acc.data_mut()) }
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
        U8BoolMut(&mut pool.is_rebalancing).set_false();
    }

    let (old_total_sol_value, inp_lst_idx) = {
        let rr =
            unsafe { RebalanceRecord::of_acc_data(abr.get(*ix_prefix.rebalance_record()).data()) }
                .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidRebalanceRecordData))?;
        (rr.old_total_sol_value, rr.dst_lst_index as usize)
    };

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

    let new_total_sol_value = {
        let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
        pool.total_sol_value
    };

    if new_total_sol_value < old_total_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    let rebalance_record_lamports = abr.get(*ix_prefix.rebalance_record()).lamports();
    if rebalance_record_lamports > 0 {
        abr.transfer_direct(
            *ix_prefix.rebalance_record(),
            *ix_prefix.pool_state(),
            rebalance_record_lamports,
        )?;
    }

    abr.get_mut(*ix_prefix.rebalance_record())
        .realloc(0, false)?;

    let rebalance_record_seeds = [
        PdaSeed::new(b"rebalance-record"),
        PdaSeed::new(&[inf1_ctl_jiminy::keys::REBALANCE_RECORD_BUMP]),
    ];
    let rebalance_record_signer = PdaSigner::new(&rebalance_record_seeds);

    let system_prog_key = *abr.get(*ix_prefix.system_program()).key();

    cpi.invoke_signed(
        abr,
        &system_prog_key,
        AssignIxData::new(&SYSTEM_PROGRAM_ID).as_buf(),
        assign_ix_account_handle_perms(
            NewAssignIxAccsBuilder::start()
                .with_assign(*ix_prefix.rebalance_record())
                .build(),
        ),
        &[rebalance_record_signer],
    )?;

    Ok(())
}

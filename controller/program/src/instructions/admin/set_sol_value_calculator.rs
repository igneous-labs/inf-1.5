use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolState,
    },
    cpi::SetSolValueCalculatorIxPreAccountHandles,
    err::Inf1CtlErr,
    instructions::{
        admin::set_sol_value_calculator::{
            NewSetSolValueCalculatorIxPreAccsBuilder, SetSolValueCalculatorIxPreAccs,
            SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};

use inf1_core::instructions::admin::set_sol_value_calculator::SetSolValueCalculatorIxAccs;
use inf1_core::instructions::sync_sol_value::SyncSolValueIxAccs;

use crate::{
    svc::lst_sync_sol_val_unchecked,
    verify::{
        verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers,
        verify_sol_value_calculator_is_program,
    },
    Cpi,
};

pub type SetSolValueCalculatorIxAccounts<'a, 'acc> = SetSolValueCalculatorIxAccs<
    AccountHandle<'acc>,
    SetSolValueCalculatorIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
>;

/// Returns (prefix, sol_val_calc_program, remaining accounts)
#[inline]
fn set_sol_value_calculator_accs_checked<'a, 'acc>(
    abr: &Abr,
    accounts: &'a [AccountHandle<'acc>],
    lst_idx: usize,
) -> Result<SetSolValueCalculatorIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = SetSolValueCalculatorIxPreAccs(*ix_prefix);
    let list = LstStatePackedList::of_acc_data(abr.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;
    let lst_state = list
        .0
        .get(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    let lst_mint_acc = abr.get(*ix_prefix.lst_mint());
    let token_prog = lst_mint_acc.owner();
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state() };
    let expected_reserves =
        create_raw_pool_reserves_addr(token_prog, &lst_state.mint, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let expected_pks = NewSetSolValueCalculatorIxPreAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_lst_mint(&lst_state.mint)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_reserves(&expected_reserves)
        .with_pool_state(&POOL_STATE_ID)
        .build();
    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    verify_signers(abr, &ix_prefix.0, &SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let (calc_prog, calc) = suf.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_sol_value_calculator_is_program(abr.get(*calc_prog))?;

    Ok(SetSolValueCalculatorIxAccounts {
        ix_prefix,
        calc_prog: *calc_prog,
        calc,
    })
}

#[inline]
pub fn process_set_sol_value_calculator(
    abr: &mut Abr,
    accounts: &[AccountHandle],
    lst_idx: usize,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let SetSolValueCalculatorIxAccounts {
        ix_prefix,
        calc_prog,
        calc,
    } = set_sol_value_calculator_accs_checked(abr, accounts, lst_idx)?;

    let calc_key = *abr.get(calc_prog).key();

    let list =
        LstStatePackedListMut::of_acc_data(abr.get_mut(*ix_prefix.lst_state_list()).data_mut())
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;
    let lst_state = list
        .0
        .get_mut(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state_mut() };

    lst_state.sol_value_calculator = calc_key;

    lst_sync_sol_val_unchecked(
        abr,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.pool_reserves())
                .build(),
            calc_prog,
            calc,
        },
        lst_idx,
    )
}

use inf1_ctl_jiminy::{
    account_utils::{
        lst_state_list_checked, pool_state_checked, pool_state_checked_mut,
        rebalance_record_checked,
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
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};

use inf1_core::instructions::{
    rebalance::end::EndRebalanceIxAccs, sync_sol_value::SyncSolValueIxAccs,
};

use crate::{
    svc::lst_sync_sol_val_unchecked,
    verify::{verify_is_rebalancing, verify_pks, verify_signers},
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

    let pool = pool_state_checked(abr.get(*ix_prefix.pool_state()))?;
    let list = lst_state_list_checked(abr.get(*ix_prefix.lst_state_list()))?;

    verify_is_rebalancing(pool)?;

    let rr = rebalance_record_checked(abr.get(*ix_prefix.rebalance_record()))?;

    let inp_lst_idx = rr.inp_lst_index as usize;
    let inp_lst_state = list
        .0
        .get(inp_lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let inp_lst_mint_acc = abr.get(*ix_prefix.inp_lst_mint());
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
        .build();
    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    verify_signers(abr, &ix_prefix.0, &END_REBALANCE_IX_PRE_IS_SIGNER.0)?;

    let (inp_calc_prog, inp_calc) = suf.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

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

    let pool_acc = abr.get_mut(*ix_prefix.pool_state());
    let pool = pool_state_checked_mut(pool_acc)?;
    U8BoolMut(&mut pool.is_rebalancing).set_false();

    let (old_total_sol_value, inp_lst_idx) = {
        let rr = rebalance_record_checked(abr.get(*ix_prefix.rebalance_record()))?;

        (rr.old_total_sol_value, rr.inp_lst_index as usize)
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
        let pool = pool_state_checked(abr.get(*ix_prefix.pool_state()))?;
        pool.total_sol_value
    };

    if new_total_sol_value < old_total_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    abr.close(*ix_prefix.rebalance_record(), *ix_prefix.pool_state())?;

    Ok(())
}

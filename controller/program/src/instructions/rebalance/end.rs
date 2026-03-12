use inf1_ctl_jiminy::{
    account_utils::{
        lst_state_list_checked, lst_state_list_get, pool_state_v2_checked,
        pool_state_v2_checked_mut, rebalance_record_checked,
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
    sync_sol_val::SyncSolVal,
    typedefs::{
        pool_sv::{PoolSvLamports, PoolSvMutRefs},
        u8bool::U8BoolMut,
    },
    yields::update::UpdateYield,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use inf1_core::instructions::{
    rebalance::end::EndRebalanceIxAccs, sync_sol_value::SyncSolValueIxAccs,
};

use crate::{
    svc::{cpi_lst_reserves_sol_val, update_lst_state_sol_val},
    utils::{accs_split_first_chunk, split_suf_accs},
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
    let (ix_prefix, suf) = accs_split_first_chunk(accounts)?;
    let ix_prefix = EndRebalanceIxPreAccs(*ix_prefix);

    let pool = pool_state_v2_checked(abr.get(*ix_prefix.pool_state()))?;
    let list = lst_state_list_checked(abr.get(*ix_prefix.lst_state_list()))?;

    verify_is_rebalancing(pool)?;

    let rr = rebalance_record_checked(abr.get(*ix_prefix.rebalance_record()))?;

    let inp_lst_idx = rr.inp_lst_index as usize;
    let inp_lst_state = lst_state_list_get(list, inp_lst_idx)?;

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

    let [(inp_calc_prog, inp_calc)] = split_suf_accs(suf, &[])?;

    verify_pks(
        abr,
        &[inp_calc_prog],
        &[&inp_lst_state.sol_value_calculator],
    )?;

    Ok(EndRebalanceIxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
    })
}

#[inline]
pub fn process_end_rebalance(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accounts: &[AccountHandle],
) -> Result<(), ProgramError> {
    let EndRebalanceIxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
    } = end_rebalance_accs_checked(abr, accounts)?;

    let (old_total_sol_value, inp_lst_idx) = {
        let rr = rebalance_record_checked(abr.get(*ix_prefix.rebalance_record()))?;
        (rr.old_total_sol_value, rr.inp_lst_index as usize)
    };

    abr.close(*ix_prefix.rebalance_record(), *ix_prefix.pool_state())?;

    let inp_lst_new = cpi_lst_reserves_sol_val(
        abr,
        cpi,
        &SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.inp_lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.inp_pool_reserves())
                .build(),
            calc_prog: *abr.get(inp_calc_prog).key(),
            calc: inp_calc,
        },
    )?;
    let inp_sol_val =
        update_lst_state_sol_val(abr, *ix_prefix.lst_state_list(), inp_lst_idx, inp_lst_new)?;

    let pool_acc = abr.get_mut(*ix_prefix.pool_state());
    let pool = pool_state_v2_checked_mut(pool_acc)?;

    U8BoolMut(&mut pool.is_rebalancing).set_false();

    let new_total_sol_value = SyncSolVal {
        lst_sol_val: inp_sol_val,
    }
    .exec(pool.total_sol_value)
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;

    if new_total_sol_value < old_total_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    let new = UpdateYield {
        new_total_sol_value,
        old: PoolSvLamports::from_pool_state_v2(pool)
            // compare against val stored in RebalanceRecord
            .with_total(old_total_sol_value),
    }
    .exec()
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;

    PoolSvMutRefs::from_pool_state_v2(pool).update(new);

    Ok(())
}

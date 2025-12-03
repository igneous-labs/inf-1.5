use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, pool_state_v2_checked},
    err::Inf1CtlErr,
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxAccs, REMOVE_LST_IX_IS_SIGNER,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID},
    pda_onchain::{
        create_raw_pool_reserves_addr, create_raw_protocol_fee_accumulator_addr, POOL_STATE_SIGNER,
        PROTOCOL_FEE_SIGNER,
    },
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_spl_token_jiminy::{
    instructions::close_account::close_account_ix_account_handle_perms,
    sanctum_spl_token_core::instructions::close_account::{
        CloseAccountIxData, NewCloseAccountIxAccsBuilder,
    },
};
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::NewTransferIxAccsBuilder;

use crate::{
    token::get_token_account_amount,
    utils::shrink_lst_state_list,
    verify::{verify_not_rebalancing_and_not_disabled_v2, verify_pks, verify_signers},
    Cpi,
};

#[inline]
pub fn process_remove_lst(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accounts: &[AccountHandle],
    lst_idx: usize,
) -> Result<(), ProgramError> {
    let accs = accounts.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = RemoveLstIxAccs(*accs);

    let list = lst_state_list_checked(abr.get(*accs.lst_state_list()))?;
    let lst_state = list
        .0
        .get(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    let lst_mint_acc = abr.get(*accs.lst_mint());
    let token_prog = *lst_mint_acc.owner();

    let expected_reserves =
        create_raw_pool_reserves_addr(&token_prog, &lst_state.mint, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;
    let expected_protocol_fee_accumulator = create_raw_protocol_fee_accumulator_addr(
        &token_prog,
        &lst_state.mint,
        &lst_state.protocol_fee_accumulator_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewRemoveLstIxAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_lst_mint(&lst_state.mint)
        .with_pool_reserves(&expected_reserves)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_protocol_fee_accumulator_auth(&PROTOCOL_FEE_ID)
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_lst_token_program(&token_prog)
        // Free account - admin can specify any account to refund rent to
        .with_refund_rent_to(abr.get(*accs.refund_rent_to()).key())
        .build();

    verify_pks(abr, &accs.0, &expected_pks.0)?;
    verify_signers(abr, &accs.0, &REMOVE_LST_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled_v2(pool)?;

    let [l, p] = [accs.pool_reserves(), accs.protocol_fee_accumulator()]
        .map(|h| get_token_account_amount(abr.get(*h)));
    let lst_balance = l?;
    let protocol_fee_accumulator_balance = p?;

    if lst_state.sol_value != 0 || lst_balance != 0 || protocol_fee_accumulator_balance != 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstStillHasValue).into());
    }

    // Close protocol fee accumulator and pool reserves ATAs
    [
        (
            *accs.protocol_fee_accumulator(),
            *accs.protocol_fee_accumulator_auth(),
            PROTOCOL_FEE_SIGNER,
        ),
        (*accs.pool_reserves(), *accs.pool_state(), POOL_STATE_SIGNER),
    ]
    .into_iter()
    .try_for_each(|(close, auth, signer)| -> Result<(), ProgramError> {
        cpi.invoke_signed(
            abr,
            &token_prog,
            CloseAccountIxData::as_buf(),
            close_account_ix_account_handle_perms(
                NewCloseAccountIxAccsBuilder::start()
                    .with_close(close)
                    .with_dst(*accs.refund_rent_to())
                    .with_auth(auth)
                    .build(),
            ),
            &[signer],
        )
    })?;

    shrink_lst_state_list(
        abr,
        &NewTransferIxAccsBuilder::start()
            .with_from(*accs.lst_state_list())
            .with_to(*accs.refund_rent_to())
            .build(),
        &Rent::get()?,
        lst_idx,
    )?;

    Ok(())
}

use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, pool_state_v2_checked},
    err::Inf1CtlErr,
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxAccs, REMOVE_LST_IX_IS_SIGNER,
    },
    pda::CONST_PDA_KEYS_OWNED,
    pda_onchain::{
        create_raw_pool_reserves_addr, create_raw_protocol_fee_accumulator_addr, POOL_STATE_SIGNER,
    },
    program_err::Inf1CtlCustomProgErr,
    token_info::{TokenInfo, TokenInfoDestr},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};
use jiminy_sysvar_rent::Rent;
use sanctum_spl_token_jiminy::{
    instructions::close_account::close_account_ix_account_handle_perms,
    sanctum_spl_token_core::instructions::close_account::{
        CloseAccountIxData, NewCloseAccountIxAccsBuilder,
    },
};
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::NewTransferIxAccsBuilder;

use crate::{
    token::get_token_account_amount,
    utils::{accs_split_first_chunk, shrink_lst_state_list},
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers},
    Cpi,
};

#[inline]
pub fn process_remove_lst(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &[AccountHandle],
    lst_idx: usize,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let (accs, _) = accs_split_first_chunk(accs)?;
    let accs = RemoveLstIxAccs(*accs);

    let list = lst_state_list_checked(abr.get(*accs.lst_state_list()))?;
    let lst_state = list
        .0
        .get(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    let lst_mint_acc = abr.get(*accs.lst_mint());
    let token_prog = *lst_mint_acc.owner();

    let token_info = TokenInfo::from_destr(TokenInfoDestr {
        program: &token_prog,
        mint: &lst_state.mint,
    });
    let expected_reserves =
        create_raw_pool_reserves_addr(&token_info, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;
    let expected_protocol_fee_accumulator = create_raw_protocol_fee_accumulator_addr(
        &token_info,
        &lst_state.protocol_fee_accumulator_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewRemoveLstIxAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_lst_mint(&lst_state.mint)
        .with_pool_reserves(&expected_reserves)
        .with_pool_state(CONST_PDA_KEYS_OWNED.pool_state())
        .with_lst_state_list(CONST_PDA_KEYS_OWNED.lst_state_list())
        .with_lst_token_program(&token_prog)
        // even though we no longer create or do anything with the protocol fee accumulator
        // we still verify their identities here to maintain consistency of interface and behaviour
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_protocol_fee_accumulator_auth(CONST_PDA_KEYS_OWNED.protocol_fee())
        // Free account - admin can specify any account to refund rent to
        .with_refund_rent_to(abr.get(*accs.refund_rent_to()).key())
        .build();

    verify_pks(abr, &accs.0, &expected_pks.0)?;
    verify_signers(abr, &accs.0, &REMOVE_LST_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let [l, p] = [accs.pool_reserves(), accs.protocol_fee_accumulator()]
        .map(|h| get_token_account_amount(abr.get(*h)));
    let lst_balance = l?;
    let protocol_fee_accumulator_balance = p?;

    if lst_state.sol_value != 0 || lst_balance != 0 || protocol_fee_accumulator_balance != 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstStillHasValue).into());
    }

    // Close pool reserves ATAs
    cpi.invoke_signed(
        abr,
        &token_prog,
        CloseAccountIxData::as_buf(),
        close_account_ix_account_handle_perms(
            NewCloseAccountIxAccsBuilder::start()
                .with_close(*accs.pool_reserves())
                .with_dst(*accs.refund_rent_to())
                .with_auth(*accs.pool_state())
                .build(),
        ),
        &[POOL_STATE_SIGNER],
    )?;

    shrink_lst_state_list(
        abr,
        &NewTransferIxAccsBuilder::start()
            .with_from(*accs.lst_state_list())
            .with_to(*accs.refund_rent_to())
            .build(),
        rent,
        lst_idx,
    )?;

    Ok(())
}

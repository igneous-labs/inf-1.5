use inf1_ctl_jiminy::{
    account_utils::pool_state_v2_checked,
    err::Inf1CtlErr,
    instructions::protocol_fee::withdraw_protocol_fees::v1::{
        NewWithdrawProtocolFeesIxAccsBuilder, WithdrawProtocolFeesIxAccs,
        WithdrawProtocolFeesIxData, WITHDRAW_PROTOCOL_FEES_IX_IS_SIGNER,
    },
    keys::{POOL_STATE_ID, PROTOCOL_FEE_ID},
    pda_onchain::{find_protocol_fee_accumulator, PROTOCOL_FEE_SIGNER},
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA, INVALID_SEEDS},
    Cpi,
};
use sanctum_spl_token_jiminy::{
    instructions::transfer::transfer_checked_ix_account_handle_perms,
    sanctum_spl_token_core::instructions::transfer::{
        NewTransferCheckedIxAccsBuilder, TransferCheckedIxData,
    },
};

use crate::{
    token::{checked_mint_of, get_token_account_amount},
    utils::accs_split_first_chunk,
    verify::{
        verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers,
        verify_tokenkeg_or_22_mint,
    },
};

type WithdrawProtocolFeesIxAccounts<'acc> = WithdrawProtocolFeesIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn withdraw_protocol_fees_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
    ix_data_no_discm: &[u8],
) -> Result<(WithdrawProtocolFeesIxAccounts<'acc>, u64), ProgramError> {
    let (accs, _) = accs_split_first_chunk(accs)?;
    let accs = WithdrawProtocolFeesIxAccs(*accs);

    let data: &[u8; 8] = ix_data_no_discm
        .try_into()
        .map_err(|_| INVALID_INSTRUCTION_DATA)?;
    let amt = WithdrawProtocolFeesIxData::parse_no_discm(data);

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;
    let mint_acc = abr.get(*accs.lst_mint());
    let token_prog = mint_acc.owner();
    let (expected_protocol_fee_accumulator, _) =
        find_protocol_fee_accumulator(token_prog, mint_acc.key()).ok_or(INVALID_SEEDS)?;

    let expected_pks = NewWithdrawProtocolFeesIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_protocol_fee_accumulator_auth(&PROTOCOL_FEE_ID)
        .with_beneficiary(&pool.protocol_fee_beneficiary)
        .with_token_program(token_prog)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        // Free: the beneficiary is entitled to all balances of all ATAs of the protocol fee PDA,
        // including tokens that are not part of the pool
        // owner = token-22 or tokenkeg checked below
        .with_lst_mint(mint_acc.key())
        // Free: the beneficiary is free to specify whatever token account to withdraw to
        // In the case of an invalid token acc, token prog transfer CPI will fail
        .with_withdraw_to(abr.get(*accs.withdraw_to()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &WITHDRAW_PROTOCOL_FEES_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    verify_tokenkeg_or_22_mint(mint_acc)?;

    let accum_bal = get_token_account_amount(abr.get(*accs.protocol_fee_accumulator()))?;
    if amt > accum_bal {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::NotEnoughFees).into());
    }

    Ok((accs, amt))
}

#[inline]
pub fn process_withdraw_protocol_fees(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &WithdrawProtocolFeesIxAccounts,
    amt: u64,
) -> Result<(), ProgramError> {
    let decimals = checked_mint_of(abr.get(*accs.lst_mint()))?.decimals();

    cpi.invoke_signed_handle(
        abr,
        *accs.token_program(),
        TransferCheckedIxData::new(amt, decimals).as_buf(),
        transfer_checked_ix_account_handle_perms(
            NewTransferCheckedIxAccsBuilder::start()
                .with_mint(*accs.lst_mint())
                .with_auth(*accs.protocol_fee_accumulator_auth())
                .with_src(*accs.protocol_fee_accumulator())
                .with_dst(*accs.withdraw_to())
                .build(),
        ),
        &[PROTOCOL_FEE_SIGNER],
    )?;

    Ok(())
}

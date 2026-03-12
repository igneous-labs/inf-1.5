use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    err::Inf1CtlErr,
    instructions::protocol_fee::withdraw_protocol_fees::v2::{
        NewWithdrawProtocolFeesV2IxAccsBuilder, WithdrawProtocolFeesV2IxAccs,
        WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
    pda_onchain::POOL_STATE_SIGNER,
    program_err::Inf1CtlCustomProgErr,
    svc::InfCalc,
    typedefs::pool_sv::PoolSvLamports,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
    Cpi,
};
use jiminy_sysvar_clock::Clock;
use sanctum_spl_token_jiminy::{
    instructions::mint_to::mint_to_ix_account_handle_perms,
    sanctum_spl_token_core::instructions::mint_to::{MintToIxData, NewMintToIxAccsBuilder},
};

use crate::{
    token::checked_mint_of,
    utils::accs_split_first_chunk,
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers},
};

type WithdrawProtocolFeesV2IxAccounts<'acc> = WithdrawProtocolFeesV2IxAccs<AccountHandle<'acc>>;

#[inline]
pub fn withdraw_protocol_fees_v2_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<WithdrawProtocolFeesV2IxAccounts<'acc>, ProgramError> {
    let (ix_prefix, _) = accs_split_first_chunk(accs)?;
    let accs = WithdrawProtocolFeesV2IxAccs(*ix_prefix);

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;
    let mint_acc = abr.get(*accs.inf_mint());

    let expected_pks = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_beneficiary(&pool.protocol_fee_beneficiary)
        .with_inf_mint(&pool.lp_token_mint)
        .with_token_program(mint_acc.owner())
        // Free: the beneficiary is free to specify whatever INF token account to withdraw to
        // In the case of an invalid INF token acc, token prog mint_to CPI will fail
        .with_withdraw_to(abr.get(*accs.withdraw_to()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    Ok(accs)
}

#[inline]
pub fn process_withdraw_protocol_fees_v2(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &WithdrawProtocolFeesV2IxAccounts,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    pool.release_yield(clock.slot)
        .map_err(Inf1CtlCustomProgErr)?;

    let protocol_fee_lamports = pool.protocol_fee_lamports;

    if protocol_fee_lamports == 0 {
        return Ok(());
    }

    let pool_lamports = PoolSvLamports::from_pool_state_v2(pool);
    let inf_token_supply = checked_mint_of(abr.get(*accs.inf_mint()))?.supply();

    let inf_calc = InfCalc {
        pool_lamports,
        mint_supply: inf_token_supply,
    };

    let inf_to_mint = *inf_calc
        .sol_to_inf(protocol_fee_lamports)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?
        .start();

    if inf_to_mint == 0 {
        return Ok(());
    }

    cpi.invoke_signed_handle(
        abr,
        *accs.token_program(),
        MintToIxData::new(inf_to_mint).as_buf(),
        mint_to_ix_account_handle_perms(
            NewMintToIxAccsBuilder::start()
                .with_auth(*accs.pool_state())
                .with_mint(*accs.inf_mint())
                .with_to(*accs.withdraw_to())
                .build(),
        ),
        &[POOL_STATE_SIGNER],
    )?;

    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    pool.protocol_fee_lamports = 0;

    Ok(())
}

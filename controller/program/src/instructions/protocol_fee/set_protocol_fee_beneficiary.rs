use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    accounts::pool_state::PoolStateV2,
    instructions::protocol_fee::set_protocol_fee_beneficiary::{
        NewSetProtocolFeeBeneficiaryIxAccsBuilder, SetProtocolFeeBeneficiaryIxAccs,
        SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::{
    utils::accs_split_first_chunk,
    verify::{verify_pks, verify_signers},
};

type SetProtocolFeeBeneficiaryIxAccounts<'acc> =
    SetProtocolFeeBeneficiaryIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_protocol_fee_beneficiary_accs_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<SetProtocolFeeBeneficiaryIxAccounts<'acc>, ProgramError> {
    let (accs, _) = accs_split_first_chunk(accs)?;
    let accs = SetProtocolFeeBeneficiaryIxAccs(*accs);

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_curr(&pool.protocol_fee_beneficiary)
        // Free: current beneficiary is free to set new beneficiary to whatever pk as pleased
        .with_new(abr.get(*accs.new()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_SIGNER.0)?;

    Ok(accs)
}

#[inline]
pub fn process_set_protocol_fee_beneficiary(
    abr: &mut Abr,
    accs: &SetProtocolFeeBeneficiaryIxAccounts,
) -> Result<(), ProgramError> {
    let new_beneficiary = *abr.get(*accs.new()).key();
    let PoolStateV2 {
        protocol_fee_beneficiary,
        ..
    } = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    *protocol_fee_beneficiary = new_beneficiary;
    Ok(())
}

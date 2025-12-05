use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    accounts::pool_state::PoolStateV2,
    err::Inf1CtlErr,
    instructions::rps::set_rps_auth::{
        NewSetRpsAuthIxAccsBuilder, SetRpsAuthIxAccs, SET_RPS_AUTH_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::{
    utils::accs_split_first_chunk,
    verify::{verify_pks, verify_signers},
};

type SetRpsAuthIxAccounts<'acc> = SetRpsAuthIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_rps_auth_accs_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<SetRpsAuthIxAccounts<'acc>, ProgramError> {
    let (ix_prefix, _) = accs_split_first_chunk(accs)?;
    let accs = SetRpsAuthIxAccs(*ix_prefix);

    let expected_pks = NewSetRpsAuthIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        // Free: check either rps auth or pool admin below
        .with_signer(abr.get(*accs.signer()).key())
        // Free: signer is free to set new RPS auth to whatever pk as pleased
        .with_new_rps_auth(abr.get(*accs.new_rps_auth()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_RPS_AUTH_IX_IS_SIGNER.0)?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let signer_pk = abr.get(*accs.signer()).key();

    if *signer_pk != pool.rps_authority && *signer_pk != pool.admin {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedSetRpsAuthoritySigner).into());
    }

    Ok(accs)
}

#[inline]
pub fn process_set_rps_auth(
    abr: &mut Abr,
    accs: &SetRpsAuthIxAccounts,
) -> Result<(), ProgramError> {
    let new_rps_auth = *abr.get(*accs.new_rps_auth()).key();
    let PoolStateV2 { rps_authority, .. } =
        pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    *rps_authority = new_rps_auth;
    Ok(())
}

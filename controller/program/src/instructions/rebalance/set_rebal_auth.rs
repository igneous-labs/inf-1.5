use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    accounts::pool_state::PoolStateV2,
    err::Inf1CtlErr,
    instructions::rebalance::set_rebal_auth::{
        NewSetRebalAuthIxAccsBuilder, SetRebalAuthIxAccs, SET_REBAL_AUTH_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};

use crate::verify::{verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers};

type SetRebalAuthIxAccounts<'acc> = SetRebalAuthIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_rebal_auth_accs_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<SetRebalAuthIxAccounts<'acc>, ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = SetRebalAuthIxAccs(*accs);

    let expected_pks = NewSetRebalAuthIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        // Free: check either rebal auth or pool admin below
        .with_signer(abr.get(*accs.signer()).key())
        // Free: signer is free to set new auth to whatever pk as pleased
        .with_new(abr.get(*accs.new()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_REBAL_AUTH_IX_IS_SIGNER.0)?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let signer_pk = abr.get(*accs.signer()).key();

    if *signer_pk != pool.rebalance_authority && *signer_pk != pool.admin {
        return Err(
            Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedSetRebalanceAuthoritySigner).into(),
        );
    }

    Ok(accs)
}

#[inline]
pub fn process_set_rebal_auth(
    abr: &mut Abr,
    accs: SetRebalAuthIxAccounts,
) -> Result<(), ProgramError> {
    let new_rebal_auth = *abr.get(*accs.new()).key();
    let PoolStateV2 {
        rebalance_authority,
        ..
    } = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    *rebalance_authority = new_rebal_auth;
    Ok(())
}

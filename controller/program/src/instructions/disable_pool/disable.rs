use inf1_ctl_jiminy::{
    account_utils::{
        disable_pool_auth_list_checked, pool_state_v2_checked, pool_state_v2_checked_mut,
    },
    accounts::{packed_list::PackedList, pool_state::PoolStateV2},
    err::Inf1CtlErr,
    instructions::disable_pool::disable::{
        DisablePoolIxAccs, NewDisablePoolIxAccsBuilder, DISABLE_POOL_IX_IS_SIGNER,
    },
    keys::{DISABLE_POOL_AUTHORITY_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::U8BoolMut,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_clock::Clock;

use crate::{
    acc_migrations::pool_state,
    verify::{verify_not_rebalancing_and_not_disabled_v2, verify_pks, verify_signers},
};

type DisablePoolIxAccounts<'acc> = DisablePoolIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn disable_pool_accs_checked<'acc>(
    abr: &mut Abr,
    accs: &[AccountHandle<'acc>],
    clock: &Clock,
) -> Result<DisablePoolIxAccounts<'acc>, ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = DisablePoolIxAccs(*accs);

    pool_state::v2::migrate_idmpt(abr.get_mut(*accs.pool_state()), clock)?;

    let signer_pk = abr.get(*accs.signer()).key();

    let expected_pks = NewDisablePoolIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_disable_pool_auth_list(&DISABLE_POOL_AUTHORITY_LIST_ID)
        // Free: either admin or disable pool auth checked below
        .with_signer(signer_pk)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &DISABLE_POOL_IX_IS_SIGNER.0)?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    verify_not_rebalancing_and_not_disabled_v2(pool)?;

    let PackedList(auths) =
        disable_pool_auth_list_checked(abr.get(*accs.disable_pool_auth_list()))?;
    if *signer_pk != pool.admin && !auths.iter().any(|pk| pk == signer_pk) {
        return Err(
            Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedDisablePoolAuthoritySigner).into(),
        );
    }

    Ok(accs)
}

#[inline]
pub fn process_disable_pool(
    abr: &mut Abr,
    accs: &DisablePoolIxAccounts,
) -> Result<(), ProgramError> {
    let PoolStateV2 { is_disabled, .. } =
        pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    U8BoolMut(is_disabled).set_true();
    Ok(())
}

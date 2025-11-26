use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    accounts::pool_state::PoolStateV2,
    err::Inf1CtlErr,
    instructions::disable_pool::enable::{
        EnablePoolIxAccs, NewEnablePoolIxAccsBuilder, ENABLE_POOL_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::{U8Bool, U8BoolMut},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};

use crate::verify::{verify_pks, verify_signers};

type EnablePoolIxAccounts<'acc> = EnablePoolIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn enable_pool_accs_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<EnablePoolIxAccounts<'acc>, ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = EnablePoolIxAccs(*accs);

    let PoolStateV2 {
        admin, is_disabled, ..
    } = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewEnablePoolIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_admin(admin)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &ENABLE_POOL_IX_IS_SIGNER.0)?;

    if !U8Bool(is_disabled).to_bool() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolEnabled).into());
    }

    Ok(accs)
}

#[inline]
pub fn process_enable_pool(abr: &mut Abr, accs: &EnablePoolIxAccounts) -> Result<(), ProgramError> {
    let PoolStateV2 { is_disabled, .. } =
        pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    U8BoolMut(is_disabled).set_false();
    Ok(())
}

use inf1_ctl_jiminy::{
    account_utils::{disable_pool_auth_list_checked, pool_state_checked},
    accounts::pool_state::PoolState,
    err::Inf1CtlErr,
    instructions::disable_pool::remove_disable_pool_auth::{
        NewRemoveDisablePoolAuthIxAccsBuilder, RemoveDisablePoolAuthIxAccs,
        RemoveDisablePoolAuthIxData, REMOVE_DISABLE_POOL_AUTH_IX_IS_SIGNER,
    },
    keys::{DISABLE_POOL_AUTHORITY_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::NewTransferIxAccsBuilder;

use crate::{
    utils::shrink_disable_pool_auth_list,
    verify::{verify_pks, verify_signers},
};

type RemoveDisablePoolAuthAccounts<'acc> = RemoveDisablePoolAuthIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn remove_disable_pool_auth_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
    data_no_discm: &[u8],
) -> Result<(RemoveDisablePoolAuthAccounts<'acc>, usize), ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = RemoveDisablePoolAuthIxAccs(*accs);

    let idx = RemoveDisablePoolAuthIxData::parse_no_discm(
        data_no_discm
            .try_into()
            .map_err(|_| INVALID_INSTRUCTION_DATA)?,
    ) as usize;

    let list = disable_pool_auth_list_checked(abr.get(*accs.disable_pool_auth_list()))?;
    let expected_remove = list.0.get(idx).ok_or(Inf1CtlCustomProgErr(
        Inf1CtlErr::InvalidDisablePoolAuthorityIndex,
    ))?;
    let signer_pk = abr.get(*accs.signer()).key();

    let expected_pks = NewRemoveDisablePoolAuthIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_disable_pool_auth_list(&DISABLE_POOL_AUTHORITY_LIST_ID)
        .with_remove(expected_remove)
        // Free: rent refund destination can be set to anything signer wants
        .with_refund_rent_to(abr.get(*accs.refund_rent_to()).key())
        // Free: signer == (pool.admin or auth being removed) checked below
        .with_signer(signer_pk)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &REMOVE_DISABLE_POOL_AUTH_IX_IS_SIGNER.0)?;

    let PoolState { admin, .. } = pool_state_checked(abr.get(*accs.pool_state()))?;
    if signer_pk != expected_remove && signer_pk != admin {
        return Err(
            Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedDisablePoolAuthoritySigner).into(),
        );
    }

    Ok((accs, idx))
}

#[inline]
pub fn process_remove_disable_pool_auth(
    abr: &mut Abr,
    accs: &RemoveDisablePoolAuthAccounts,
    idx: usize,
) -> Result<(), ProgramError> {
    shrink_disable_pool_auth_list(
        abr,
        &NewTransferIxAccsBuilder::start()
            .with_from(*accs.disable_pool_auth_list())
            .with_to(*accs.refund_rent_to())
            .build(),
        &Rent::get()?,
        idx,
    )
}

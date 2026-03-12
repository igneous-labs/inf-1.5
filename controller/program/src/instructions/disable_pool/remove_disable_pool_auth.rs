use inf1_ctl_jiminy::{
    account_utils::{
        disable_pool_auth_list_checked, disable_pool_auth_list_get, pool_state_v2_checked,
    },
    accounts::pool_state::PoolStateV2,
    instructions::disable_pool::remove_disable_pool_auth::{
        NewRemoveDisablePoolAuthIxAccsBuilder, RemoveDisablePoolAuthIxAccs,
        RemoveDisablePoolAuthIxData, REMOVE_DISABLE_POOL_AUTH_IX_IS_SIGNER,
    },
    keys::{DISABLE_POOL_AUTHORITY_LIST_ID, POOL_STATE_ID},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA},
};
use jiminy_sysvar_rent::Rent;
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::NewTransferIxAccsBuilder;

use crate::{
    utils::{accs_split_first_chunk, shrink_disable_pool_auth_list},
    verify::{verify_pks, verify_signers},
};

type RemoveDisablePoolAuthAccounts<'acc> = RemoveDisablePoolAuthIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn remove_disable_pool_auth_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
    data_no_discm: &[u8],
) -> Result<(RemoveDisablePoolAuthAccounts<'acc>, usize), ProgramError> {
    let (accs, _) = accs_split_first_chunk(accs)?;
    let accs = RemoveDisablePoolAuthIxAccs(*accs);

    let idx = RemoveDisablePoolAuthIxData::parse_no_discm(
        data_no_discm
            .try_into()
            .map_err(|_| INVALID_INSTRUCTION_DATA)?,
    ) as usize;

    let list = disable_pool_auth_list_checked(abr.get(*accs.disable_pool_auth_list()))?;
    let expected_remove = disable_pool_auth_list_get(list, idx)?;
    let PoolStateV2 { admin, .. } = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewRemoveDisablePoolAuthIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_disable_pool_auth_list(&DISABLE_POOL_AUTHORITY_LIST_ID)
        .with_remove(expected_remove)
        .with_signer(admin)
        // Free: rent refund destination can be set to anything admin wants
        .with_refund_rent_to(abr.get(*accs.refund_rent_to()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &REMOVE_DISABLE_POOL_AUTH_IX_IS_SIGNER.0)?;

    Ok((accs, idx))
}

#[inline]
pub fn process_remove_disable_pool_auth(
    abr: &mut Abr,
    accs: &RemoveDisablePoolAuthAccounts,
    idx: usize,
    rent: &Rent,
) -> Result<(), ProgramError> {
    shrink_disable_pool_auth_list(
        abr,
        &NewTransferIxAccsBuilder::start()
            .with_from(*accs.disable_pool_auth_list())
            .with_to(*accs.refund_rent_to())
            .build(),
        rent,
        idx,
    )
}

use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, lst_state_list_get, pool_state_v2_checked},
    instructions::{
        admin::lst_input::{
            disable::{NewDisableLstInputIxAccsBuilder, DISABLE_LST_INPUT_IX_IS_SIGNER},
            SetLstInputIxAccs,
        },
        generic::u32_ix_data_parse_no_discm,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::lst_state::LstState,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::{
    utils::{accs_split_first_chunk, ix_data_as_arr},
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers},
};

#[inline]
pub fn set_lst_input_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
    data_no_discm: &[u8],
) -> Result<(SetLstInputIxAccs<AccountHandle<'acc>>, usize), ProgramError> {
    let (accs, _) = accs_split_first_chunk(accs)?;
    let accs = SetLstInputIxAccs(*accs);

    let idx = u32_ix_data_parse_no_discm(ix_data_as_arr(data_no_discm)?) as usize;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;
    let list = lst_state_list_checked(abr.get(*accs.lst_state_list()))?;
    let LstState { mint, .. } = lst_state_list_get(list, idx)?;

    let expected_pks = NewDisableLstInputIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_admin(&pool.admin)
        .with_lst_mint(mint)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &DISABLE_LST_INPUT_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    Ok((accs, idx))
}

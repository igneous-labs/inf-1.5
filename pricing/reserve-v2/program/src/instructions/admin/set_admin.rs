use inf1_pp_reserve_v2_core::{
    instructions::admin::{
        common::{AdminIxPreAccs, AdminIxPreAccsDestr, ADMIN_IX_PRE_IS_SIGNER},
        set_admin::{SetAdminIxAccs, SetAdminIxAccsGen},
    },
    pda::CONST_PDA_KEYS_OWNED,
};
use inf1_pp_reserve_v2_jiminy::account_utils::{pricing_state_checked, pricing_state_checked_mut};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::utils::{asfc, verify_pks, verify_signers};

pub type SetAdminIxAccHandles<'a> = SetAdminIxAccsGen<AccountHandle<'a>>;

pub fn set_admin_accs_checked<'acc>(
    abr: &Abr,
    accounts: &[AccountHandle<'acc>],
) -> Result<SetAdminIxAccHandles<'acc>, ProgramError> {
    let (pre_accs, rest) = asfc(accounts)?;
    let pre = AdminIxPreAccs(*pre_accs);
    let ([new_admin], _) = asfc(rest)?;

    let (expected_admin, _) = pricing_state_checked(abr.get(*pre.pricing_state()))?;
    verify_pks(
        abr,
        &pre.0,
        &AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
            pricing_state: CONST_PDA_KEYS_OWNED.pricing_state(),
            admin: expected_admin,
        })
        .0,
    )?;

    verify_signers(abr, &pre.0, &ADMIN_IX_PRE_IS_SIGNER.0)?;

    Ok(SetAdminIxAccs {
        pre,
        new_admin: *new_admin,
    })
}

pub fn process_set_admin<'acc>(
    abr: &mut Abr,
    accs: SetAdminIxAccHandles<'acc>,
) -> Result<(), ProgramError> {
    let new_admin = *abr.get(accs.new_admin).key();
    let pricing_state = abr.get_mut(*accs.pre.pricing_state());
    let (admin, _) = pricing_state_checked_mut(pricing_state)?;
    *admin = new_admin;
    Ok(())
}

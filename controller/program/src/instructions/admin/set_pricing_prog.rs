use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    instructions::admin::set_pricing_prog::{
        NewSetPricingProgIxAccsBuilder, SetPricingProgIxAccs, SET_PRICING_PROG_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};

use crate::verify::{
    verify_not_rebalancing_and_not_disabled_v2, verify_pks, verify_pricing_program_is_program,
    verify_signers,
};

type SetPricingProgIxAccounts<'acc> = SetPricingProgIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_pricing_prog_accs_checked<'acc>(
    abr: &mut Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<SetPricingProgIxAccounts<'acc>, ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = SetPricingProgIxAccs(*accs);

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;
    let new_pp = abr.get(*accs.new());

    let expected_pks = NewSetPricingProgIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_admin(&pool.admin)
        // Free: current admin is free to set new pricing program to whatever program as pleased
        .with_new(new_pp.key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_PRICING_PROG_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled_v2(pool)?;

    verify_pricing_program_is_program(new_pp)?;

    Ok(accs)
}

#[inline]
pub fn process_set_pricing_prog(
    abr: &mut Abr,
    accs: SetPricingProgIxAccounts,
) -> Result<(), ProgramError> {
    let new_pp = *abr.get(*accs.new()).key();
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    pool.pricing_program = new_pp;
    Ok(())
}

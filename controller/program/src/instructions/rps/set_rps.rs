use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    instructions::rps::set_rps::{
        NewSetRpsIxAccsBuilder, SetRpsIxAccs, SetRpsIxData, SET_RPS_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    typedefs::{rps::Rps, uq0f63::UQ0F63},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA},
};
use jiminy_sysvar_clock::Clock;

use crate::{
    utils::{accs_split_first_chunk, ix_data_as_arr},
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers},
};

type SetRpsIxAccounts<'acc> = SetRpsIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_rps_checked<'acc>(
    abr: &mut Abr,
    accs: &[AccountHandle<'acc>],
    ix_data_no_discm: &[u8],
) -> Result<(SetRpsIxAccounts<'acc>, Rps), ProgramError> {
    let (ix_prefix, _) = accs_split_first_chunk(accs)?;
    let accs = SetRpsIxAccs(*ix_prefix);

    let new_rps_raw = SetRpsIxData::parse_no_discm(ix_data_as_arr(ix_data_no_discm)?);

    // Validate raw RPS value
    let uq_rps = UQ0F63::new(new_rps_raw).map_err(|_| INVALID_INSTRUCTION_DATA)?;
    let new_rps = Rps::new(uq_rps).map_err(|_| INVALID_INSTRUCTION_DATA)?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewSetRpsIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_rps_auth(&pool.rps_authority)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_RPS_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    Ok((accs, new_rps))
}

#[inline]
pub fn process_set_rps(
    abr: &mut Abr,
    accs: &SetRpsIxAccounts,
    new_rps: Rps,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    pool.release_yield(clock.slot)
        .map_err(Inf1CtlCustomProgErr)?;

    pool.rps = *new_rps.as_raw();

    Ok(())
}

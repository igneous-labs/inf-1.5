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
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_clock::Clock;

use crate::verify::{verify_pks, verify_signers};

type SetRpsIxAccounts<'acc> = SetRpsIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_rps_checked<'acc>(
    abr: &mut Abr,
    accs: &[AccountHandle<'acc>],
    ix_data_no_discm: &[u8],
) -> Result<(SetRpsIxAccounts<'acc>, u64), ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = SetRpsIxAccs(*accs);

    let data: &[u8; 8] = ix_data_no_discm
        .try_into()
        .map_err(|_| INVALID_INSTRUCTION_DATA)?;
    let new_rps_raw = SetRpsIxData::parse_no_discm(data);

    // Validate raw RPS value
    let uq = UQ0F63::new(new_rps_raw).map_err(|_| INVALID_INSTRUCTION_DATA)?;
    let _new_rps = Rps::new(uq).map_err(|_| INVALID_INSTRUCTION_DATA)?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewSetRpsIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_rps_auth(&pool.rps_authority)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_RPS_IX_IS_SIGNER.0)?;

    Ok((accs, new_rps_raw))
}

#[inline]
pub fn process_set_rps(
    abr: &mut Abr,
    accs: &SetRpsIxAccounts,
    new_rps: u64,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    pool.release_yield(clock.slot)
        .map_err(Inf1CtlCustomProgErr)?;

    pool.rps = new_rps;

    Ok(())
}

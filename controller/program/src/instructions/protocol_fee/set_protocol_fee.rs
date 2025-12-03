use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    accounts::pool_state::PoolStateV2,
    err::Inf1CtlErr,
    instructions::protocol_fee::set_protocol_fee::{
        NewSetProtocolFeeIxAccsBuilder, SetProtocolFeeIxAccs, SetProtocolFeeIxData,
        SET_PROTOCOL_FEE_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    typedefs::fee_nanos::FeeNanos,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};

use crate::verify::{verify_not_rebalancing_and_not_disabled_v2, verify_pks, verify_signers};

type SetProtocolFeeIxAccounts<'acc> = SetProtocolFeeIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_protocol_fee_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
    ix_data_no_discm: &[u8],
) -> Result<(SetProtocolFeeIxAccounts<'acc>, u32), ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = SetProtocolFeeIxAccs(*accs);

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewSetProtocolFeeIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_admin(&pool.admin)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_PROTOCOL_FEE_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled_v2(pool)?;

    let protocol_fee_nanos = SetProtocolFeeIxData::parse_no_discm(
        ix_data_no_discm
            .first_chunk()
            .ok_or(INVALID_INSTRUCTION_DATA)?,
    );

    FeeNanos::new(protocol_fee_nanos).map_err(|_| Inf1CtlCustomProgErr(Inf1CtlErr::FeeTooHigh))?;

    Ok((accs, protocol_fee_nanos))
}

#[inline]
pub fn process_set_protocol_fee(
    abr: &mut Abr,
    accs: &SetProtocolFeeIxAccounts,
    protocol_fee_nanos: u32,
) -> Result<(), ProgramError> {
    let PoolStateV2 {
        protocol_fee_nanos: pool_protocol_fee_nanos,
        ..
    } = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;

    *pool_protocol_fee_nanos = protocol_fee_nanos;

    Ok(())
}

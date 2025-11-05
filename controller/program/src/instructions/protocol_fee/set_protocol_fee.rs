use inf1_ctl_jiminy::{
    account_utils::{pool_state_checked, pool_state_checked_mut},
    accounts::pool_state::PoolState,
    instructions::protocol_fee::set_protocol_fee::{
        NewSetProtocolFeeIxAccsBuilder, SetProtocolFeeIxAccs, SetProtocolFeeIxArgs,
        SetProtocolFeeIxData, SET_PROTOCOL_FEE_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};

use crate::verify::{
    verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers, verify_valid_fee_bps,
};

type SetProtocolFeeIxAccounts<'acc> = SetProtocolFeeIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_protocol_fee_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
    ix_data_no_discm: &[u8],
) -> Result<(SetProtocolFeeIxAccounts<'acc>, SetProtocolFeeIxArgs), ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = SetProtocolFeeIxAccs(*accs);

    let pool = pool_state_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewSetProtocolFeeIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_admin(&pool.admin)
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_PROTOCOL_FEE_IX_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let SetProtocolFeeIxArgs {
        trading_bps,
        lp_bps,
    } = SetProtocolFeeIxData::parse_no_discm(ix_data_no_discm).ok_or(INVALID_INSTRUCTION_DATA)?;

    [trading_bps, lp_bps]
        .into_iter()
        .try_for_each(|bps| bps.map_or_else(|| Ok(()), verify_valid_fee_bps))?;

    Ok((
        accs,
        SetProtocolFeeIxArgs {
            trading_bps,
            lp_bps,
        },
    ))
}

#[inline]
pub fn process_set_protocol_fee(
    abr: &mut Abr,
    accs: &SetProtocolFeeIxAccounts,
    SetProtocolFeeIxArgs {
        trading_bps,
        lp_bps,
    }: &SetProtocolFeeIxArgs,
) -> Result<(), ProgramError> {
    let PoolState {
        trading_protocol_fee_bps,
        lp_protocol_fee_bps,
        ..
    } = pool_state_checked_mut(abr.get_mut(*accs.pool_state()))?;

    [
        (trading_bps, trading_protocol_fee_bps),
        (lp_bps, lp_protocol_fee_bps),
    ]
    .into_iter()
    .for_each(|(new, refr)| {
        if let Some(new) = new {
            *refr = *new;
        }
    });

    Ok(())
}

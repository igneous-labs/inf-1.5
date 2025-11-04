use inf1_ctl_jiminy::keys::SYS_PROG_ID;
use jiminy_cpi::{
    account::Abr,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use jiminy_entrypoint::account::AccountHandle;
use jiminy_sysvar_rent::Rent;
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::{
    NewTransferIxAccsBuilder, TransferIxAccs, TransferIxData,
};

use crate::Cpi;

pub fn pay_for_rent_exempt_shortfall(
    abr: &mut Abr,
    cpi: &mut Cpi,
    handles: TransferIxAccs<AccountHandle>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let to_acc = abr.get(*handles.to());
    let lamports_shortfall = rent
        .min_balance(to_acc.data_len())
        .saturating_sub(to_acc.lamports());

    if lamports_shortfall > 0 {
        cpi.invoke_fwd(
            abr,
            &SYS_PROG_ID,
            TransferIxData::new(lamports_shortfall).as_buf(),
            NewTransferIxAccsBuilder::start()
                .with_from(*handles.from())
                .with_to(*handles.to())
                .build()
                .0,
        )?;
    }

    Ok(())
}

/// Refunds excess lamports from `from` to `to` after account reallocation
pub fn refund_excess_rent(
    abr: &mut Abr,
    handles: TransferIxAccs<AccountHandle>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let from_acc = abr.get(*handles.from());
    let lamports_surplus = from_acc
        .lamports()
        .checked_sub(rent.min_balance(from_acc.data_len()))
        .ok_or(INVALID_ACCOUNT_DATA)?;
    if lamports_surplus > 0 {
        abr.transfer_direct(*handles.from(), *handles.to(), lamports_surplus)?;
    }

    Ok(())
}

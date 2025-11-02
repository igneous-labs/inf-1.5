use inf1_ctl_jiminy::{
    instructions::admin::{add_lst::AddLstIxAccs, remove_lst::RemoveLstIxAccs},
    keys::SYS_PROG_ID,
};
use jiminy_cpi::{
    account::Abr,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use jiminy_entrypoint::account::AccountHandle;
use jiminy_sysvar_rent::Rent;
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::{
    NewTransferIxAccsBuilder, TransferIxData,
};

use crate::Cpi;

pub fn pay_for_rent_exempt_shortfall(
    abr: &mut Abr,
    cpi: &mut Cpi,
    handles: AddLstIxAccs<AccountHandle>,
    rent: Rent,
) -> Result<(), ProgramError> {
    let lst_state_list_acc = abr.get(*handles.lst_state_list());
    let lamports_shortfall = rent
        .min_balance(lst_state_list_acc.data_len())
        .saturating_sub(lst_state_list_acc.lamports());

    if lamports_shortfall > 0 {
        cpi.invoke_fwd(
            abr,
            &SYS_PROG_ID,
            TransferIxData::new(lamports_shortfall).as_buf(),
            NewTransferIxAccsBuilder::start()
                .with_from(*handles.payer())
                .with_to(*handles.lst_state_list())
                .build()
                .0,
        )?;
    }

    Ok(())
}

pub fn refund_excess_rent(
    abr: &mut Abr,
    handles: RemoveLstIxAccs<AccountHandle>,
    rent: Rent,
) -> Result<(), ProgramError> {
    let lst_state_list_acc = abr.get(*handles.lst_state_list());
    let lamports_surplus = lst_state_list_acc
        .lamports()
        .checked_sub(rent.min_balance(lst_state_list_acc.data_len()))
        .ok_or(INVALID_ACCOUNT_DATA)?;
    if lamports_surplus > 0 {
        abr.transfer_direct(
            *handles.lst_state_list(),
            *handles.refund_rent_to(),
            lamports_surplus,
        )?;
    }

    Ok(())
}

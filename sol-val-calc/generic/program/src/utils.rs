use inf1_svc_generic_jiminy::keys::GLOBAL_CONST_KEYS_OWNED;
use jiminy_account::{Abr, AccountHandle};
use jiminy_cpi::{
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
    Cpi,
};
use jiminy_sysvar_rent::Rent;
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::{
    TransferIxAccs, TransferIxData,
};

#[inline]
pub fn ix_data_as_arr<const N: usize>(ix_data: &[u8]) -> Result<&[u8; N], ProgramError> {
    Ok(ix_data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?)
}

#[inline]
pub fn accs_split_first_chunk<'a, 'acc, const N: usize>(
    accs: &'a [AccountHandle<'acc>],
) -> Result<(&'a [AccountHandle<'acc>; N], &'a [AccountHandle<'acc>]), ProgramError> {
    accs.split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS.into())
}

#[inline]
pub fn pay_for_rent_exempt_shortfall(
    abr: &mut Abr,
    cpi: &mut Cpi,
    handles: &TransferIxAccs<AccountHandle>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    let to_acc = abr.get(*handles.to());
    let lamports_shortfall = rent
        .min_balance(to_acc.data_len())
        .saturating_sub(to_acc.lamports());

    if lamports_shortfall > 0 {
        cpi.invoke_fwd(
            abr,
            GLOBAL_CONST_KEYS_OWNED.sys_prog(),
            TransferIxData::new(lamports_shortfall).as_buf(),
            handles.0,
        )?;
    }

    Ok(())
}

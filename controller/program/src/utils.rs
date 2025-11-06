use inf1_ctl_jiminy::{keys::SYS_PROG_ID, pda_onchain::DISABLE_POOL_AUTH_LIST_SIGNER, ID};
use jiminy_cpi::{
    account::Abr,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use jiminy_entrypoint::account::AccountHandle;
use jiminy_pda::PdaSigner;
use jiminy_sysvar_rent::Rent;
use sanctum_system_jiminy::{
    instructions::assign::assign_invoke_signed,
    sanctum_system_core::instructions::{
        assign::NewAssignIxAccsBuilder,
        transfer::{TransferIxAccs, TransferIxData},
    },
};

use crate::Cpi;

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
            &SYS_PROG_ID,
            TransferIxData::new(lamports_shortfall).as_buf(),
            handles.0,
        )?;
    }

    Ok(())
}

/// Refunds excess lamports from `from` to `to` after account reallocation
#[inline]
pub fn refund_excess_rent(
    abr: &mut Abr,
    handles: &TransferIxAccs<AccountHandle>,
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

/// Extends a [`inf1_ctl_jiminy::accounts::packed_list::PackedList`]
/// by 1, returning a mut reference to the new entry at the end of the list.
///
/// Assigns the list PDA to controller program if it isnt already owned by us
///
/// # Generics
/// - `T` type of `PackedList`` element
///
/// # Params
/// - `accs`. `from` should be rent payer, `to` should be list PDA being extended
#[inline]
fn extend_packed_list_pda<T>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &TransferIxAccs<AccountHandle>,
    signer: PdaSigner,
    rent: &Rent,
) -> Result<(), ProgramError> {
    if abr.get(*accs.to()).data_len() == 0 {
        assign_invoke_signed(
            abr,
            cpi,
            NewAssignIxAccsBuilder::start()
                .with_assign(*accs.to())
                .build(),
            &ID,
            &[signer],
        )?;
    }
    let list_acc = abr.get_mut(*accs.to());
    list_acc.grow_by(core::mem::size_of::<T>(), false)?;
    pay_for_rent_exempt_shortfall(abr, cpi, accs, rent)?;
    Ok(())
}

#[inline]
pub fn extend_disable_pool_auth_list(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &TransferIxAccs<AccountHandle>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    extend_packed_list_pda::<[u8; 32]>(abr, cpi, accs, DISABLE_POOL_AUTH_LIST_SIGNER, rent)
}

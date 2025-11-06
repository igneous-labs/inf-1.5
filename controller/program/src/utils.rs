use core::mem::size_of;

use inf1_ctl_jiminy::{
    keys::SYS_PROG_ID, pda_onchain::DISABLE_POOL_AUTHORITY_LIST_SIGNER,
    typedefs::lst_state::LstState, ID,
};
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

/// Extends a [`inf1_ctl_jiminy::accounts::packed_list::PackedList`] by 1
///
/// Assigns the list PDA to controller program if data is empty
/// (should mean owner = system program)
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
    rent: &Rent,
    signer: PdaSigner,
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
    list_acc.grow_by(size_of::<T>(), false)?;
    pay_for_rent_exempt_shortfall(abr, cpi, accs, rent)?;
    Ok(())
}

// TODO: extend_lst_state_list + refactor

#[inline]
pub fn extend_disable_pool_auth_list(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &TransferIxAccs<AccountHandle>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    extend_packed_list_pda::<[u8; 32]>(abr, cpi, accs, rent, DISABLE_POOL_AUTHORITY_LIST_SIGNER)
}

/// Inverse of [`extend_disable_pool_auth_list`]
///
/// Removes the given index entry from the list, shrinking it down by 1
///
/// Closes the account, returning it to system program, if its empty
///
/// # Generics
/// - `T` type of `PackedList`` element
///
/// # Params
/// - `accs`. `from` should be the list PDA being shrunk, `to` should be destination to refund rent to
#[inline]
fn shrink_packed_list_pda<T>(
    abr: &mut Abr,
    accs: &TransferIxAccs<AccountHandle>,
    rent: &Rent,
    idx: usize,
) -> Result<(), ProgramError> {
    let elem_sz = size_of::<T>();
    let list_acc = abr.get_mut(*accs.from());

    let elem_byte_start = idx.checked_mul(elem_sz).ok_or(INVALID_ACCOUNT_DATA)?;
    let elem_byte_end = elem_byte_start
        .checked_add(elem_sz)
        .ok_or(INVALID_ACCOUNT_DATA)?;

    list_acc
        .data_mut()
        .copy_within(elem_byte_end.., elem_byte_start);
    list_acc.shrink_by(elem_sz)?;

    let new_acc_len = list_acc.data_len();
    if new_acc_len == 0 {
        abr.close(*accs.from(), *accs.to())?;
    } else {
        refund_excess_rent(abr, accs, rent)?;
    }

    Ok(())
}

/// `accs`
/// - `from` disable_pool_auth_list_pda
/// - `to` refund_rent_to
#[inline]
pub fn shrink_disable_pool_auth_list(
    abr: &mut Abr,
    accs: &TransferIxAccs<AccountHandle>,
    rent: &Rent,
    idx: usize,
) -> Result<(), ProgramError> {
    shrink_packed_list_pda::<[u8; 32]>(abr, accs, rent, idx)
}

/// `accs`
/// - `from` lst_state_list_pda
/// - `to` refund_rent_to
#[inline]
pub fn shrink_lst_state_list(
    abr: &mut Abr,
    accs: &TransferIxAccs<AccountHandle>,
    rent: &Rent,
    idx: usize,
) -> Result<(), ProgramError> {
    shrink_packed_list_pda::<LstState>(abr, accs, rent, idx)
}

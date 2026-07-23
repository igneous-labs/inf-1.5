use inf1_pp_reserve_v2_core::{
    accounts::{pricing_state_of_acc_data, pricing_state_of_acc_data_mut},
    typedefs::{FeeEntryList, FeeEntryListMut},
};
use jiminy_account::Account;
use jiminy_program_error::{ProgramError, INVALID_ACCOUNT_DATA};

#[inline]
pub fn pricing_state_checked(acc: &Account) -> Result<(&[u8; 32], FeeEntryList<'_>), ProgramError> {
    pricing_state_of_acc_data(acc.data()).ok_or(INVALID_ACCOUNT_DATA.into())
}

#[inline]
pub fn pricing_state_checked_mut(
    acc: &mut Account,
) -> Result<(&mut [u8; 32], FeeEntryListMut<'_>), ProgramError> {
    pricing_state_of_acc_data_mut(acc.data_mut()).ok_or(INVALID_ACCOUNT_DATA.into())
}

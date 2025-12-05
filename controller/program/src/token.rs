use jiminy_cpi::{
    account::Account,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::{
    account::{RawTokenAccount, TokenAccount},
    mint::{Mint, RawMint},
};

#[inline]
pub fn get_token_account_amount(acc: &Account) -> Result<u64, ProgramError> {
    Ok(RawTokenAccount::of_acc_data(acc.data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?)
}

/// `_checked` because it also verifies that the acc is properly initialized
#[inline]
pub fn checked_mint_of(acc: &Account) -> Result<Mint<'_>, ProgramError> {
    Ok(RawMint::of_acc_data(acc.data())
        .and_then(Mint::try_from_raw)
        .ok_or(INVALID_ACCOUNT_DATA)?)
}

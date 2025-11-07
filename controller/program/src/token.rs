use jiminy_cpi::program_error::{ProgramError, INVALID_ACCOUNT_DATA};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::{
    RawTokenAccount, TokenAccount,
};

#[inline]
pub fn get_token_account_amount(token_acc_data: &[u8]) -> Result<u64, ProgramError> {
    Ok(RawTokenAccount::of_acc_data(token_acc_data)
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?)
}

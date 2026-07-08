use jiminy_account::AccountHandle;
use jiminy_cpi::program_error::{ProgramError, INVALID_INSTRUCTION_DATA, NOT_ENOUGH_ACCOUNT_KEYS};

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

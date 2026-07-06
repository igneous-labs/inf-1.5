use jiminy_cpi::program_error::{ProgramError, INVALID_INSTRUCTION_DATA};

#[inline]
pub fn ix_data_as_arr<const N: usize>(ix_data: &[u8]) -> Result<&[u8; N], ProgramError> {
    Ok(ix_data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?)
}

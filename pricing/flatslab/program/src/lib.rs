use inf1_pp_core::instructions::{
    price::{exact_in::PRICE_EXACT_IN_IX_DISCM, exact_out::PRICE_EXACT_OUT_IX_DISCM},
    IxArgs,
};
use jiminy_entrypoint::{
    program_entrypoint,
    program_error::{BuiltInProgramError, ProgramError},
};

use crate::instructions::pricing::{
    pricing_accs_checked, process_price_exact_in, process_price_exact_out,
};

mod err;
mod instructions;
mod utils;

// Re-exports for integration tests
pub use err::*;

const MAX_ACCS: usize = 4;

type Accounts<'account> = jiminy_entrypoint::account::Accounts<'account, MAX_ACCS>;

program_entrypoint!(process_ix, MAX_ACCS);

const INVALID_IX_DATA_ERR: ProgramError =
    ProgramError::from_builtin(BuiltInProgramError::InvalidInstructionData);

fn process_ix(
    accounts: &mut Accounts,
    data: &[u8],
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    match data.split_first().ok_or(INVALID_IX_DATA_ERR)? {
        (&PRICE_EXACT_IN_IX_DISCM, data) => {
            let (pre, suf) = pricing_accs_checked(accounts)?;
            let args = IxArgs::parse(data.try_into().map_err(|_e| INVALID_IX_DATA_ERR)?);
            process_price_exact_in(accounts, &pre, &suf, args)
        }
        (&PRICE_EXACT_OUT_IX_DISCM, data) => {
            let (pre, suf) = pricing_accs_checked(accounts)?;
            let args = IxArgs::parse(data.try_into().map_err(|_e| INVALID_IX_DATA_ERR)?);
            process_price_exact_out(accounts, &pre, &suf, args)
        }
        _ => Err(INVALID_IX_DATA_ERR),
    }
}

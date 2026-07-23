use inf1_pp_reserve_v2_core::instructions::init::INIT_IX_DISCM;
use jiminy_cpi::account::{Abr, AccountHandle};
use jiminy_entrypoint::{
    program_entrypoint,
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA},
};

use crate::instructions::init::{init_accs_checked, process_init};

mod instructions;
mod utils;

// Re-exports for integration tests
pub use inf1_pp_reserve_v2_jiminy::program_err::*;
pub use utils::*;

const MAX_ACCS: usize = 5;

program_entrypoint!(process_ix, MAX_ACCS);

fn process_ix(
    abr: &mut Abr,
    accounts: &[AccountHandle<'_>],
    data: &[u8],
    prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    match data.split_first().ok_or(INVALID_INSTRUCTION_DATA)? {
        (&INIT_IX_DISCM, _data) => {
            let accs = init_accs_checked(abr, accounts)?;
            process_init(abr, accs, prog_id)
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

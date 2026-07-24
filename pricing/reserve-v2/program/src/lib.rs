use inf1_pp_reserve_v2_core::instructions::{
    admin::set_admin::SET_ADMIN_IX_DISCM, init::INIT_IX_DISCM,
};
use jiminy_cpi::account::{Abr, AccountHandle};
use jiminy_entrypoint::{
    program_entrypoint,
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA},
};

use crate::instructions::{
    admin::set_admin::{process_set_admin, set_admin_accs_checked},
    init::{init_accs_checked, process_init},
};

mod instructions;
mod utils;

const MAX_ACCS: usize = 5;

/// Max CPI accounts needed across all instructions
const MAX_CPI_ACCS: usize = 2;

pub type Cpi = jiminy_cpi::Cpi<MAX_CPI_ACCS>;

program_entrypoint!(process_ix, MAX_ACCS);

fn process_ix(
    abr: &mut Abr,
    accounts: &[AccountHandle<'_>],
    data: &[u8],
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    match data.split_first().ok_or(INVALID_INSTRUCTION_DATA)? {
        // Admin ixs
        (&SET_ADMIN_IX_DISCM, _data) => {
            let accs = set_admin_accs_checked(abr, accounts)?;
            process_set_admin(abr, accs)
        }
        // Init
        (&INIT_IX_DISCM, _data) => {
            let accs = init_accs_checked(abr, accounts)?;
            process_init(abr, accs)
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

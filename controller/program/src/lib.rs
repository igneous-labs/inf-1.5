#![allow(unexpected_cfgs)]

use std::alloc::Layout;

use inf1_ctl_jiminy::instructions::{
    admin::set_sol_value_calculator::{SetSolValueCalculatorIxData, SET_SOL_VALUE_CALC_IX_DISCM},
    sync_sol_value::{SyncSolValueIxData, SYNC_SOL_VALUE_IX_DISCM},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::INVALID_INSTRUCTION_DATA,
};
use jiminy_entrypoint::{
    allocator::Allogator, default_panic_handler, program_entrypoint, program_error::ProgramError,
};
use jiminy_log::sol_log;

use crate::instructions::{
    set_sol_value_calculator::process_set_sol_value_calculator,
    sync_sol_value::process_sync_sol_value,
};

mod instructions;
mod svc;
mod verify;

const MAX_ACCS: usize = 64;

/// Ensure no pricing program or sol value calculator programs require
/// more than this number of accounts for CPI
const MAX_CPI_ACCS: usize = 48;

type Cpi = jiminy_cpi::Cpi<MAX_CPI_ACCS>;

// Compile-time allocation of Cpi buffer

const CONST_ALLOCS: (Allogator, *mut Cpi) = const {
    let (a, cpi_ptr) = Allogator::new().const_alloc(Layout::new::<Cpi>());
    (a, cpi_ptr.cast::<Cpi>())
};

#[cfg(target_os = "solana")]
#[global_allocator]
static ALLOCATOR: Allogator = CONST_ALLOCS.0;

const CPI_PTR: *mut Cpi = CONST_ALLOCS.1;

default_panic_handler!();
program_entrypoint!(process_ix, MAX_ACCS);

#[inline]
fn process_ix(
    abr: &mut Abr,
    accounts: &[AccountHandle<'_>],
    data: &[u8],
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    // Safety:
    // - even tho ptr is pointing to uninitialized memory,
    //   Cpi type is just wrapper around MaybeUninit::uninit()
    //   (might still be UB, idk).
    //   `as_uninit_mut` only available in nightly.
    let cpi: &'static mut Cpi = unsafe { CPI_PTR.as_mut().unwrap_unchecked() };

    match data.split_first().ok_or(INVALID_INSTRUCTION_DATA)? {
        (&SYNC_SOL_VALUE_IX_DISCM, data) => {
            sol_log("SyncSolValue");
            let lst_idx = SyncSolValueIxData::parse_no_discm(
                data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?,
            ) as usize;
            process_sync_sol_value(abr, accounts, lst_idx, cpi)
        }
        (&SET_SOL_VALUE_CALC_IX_DISCM, data) => {
            sol_log("SetSolValueCalculator");
            let lst_idx = SetSolValueCalculatorIxData::parse_no_discm(
                data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?,
            ) as usize;
            process_set_sol_value_calculator(abr, accounts, lst_idx, cpi)
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

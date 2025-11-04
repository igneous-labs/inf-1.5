#![allow(unexpected_cfgs)]

use std::alloc::Layout;

use inf1_ctl_jiminy::instructions::{
    admin::{
        add_lst::ADD_LST_IX_DISCM,
        remove_lst::{RemoveLstIxData, REMOVE_LST_IX_DISCM},
        set_admin::SET_ADMIN_IX_DISCM,
        set_pricing_prog::SET_PRICING_PROG_IX_DISCM,
        set_sol_value_calculator::{SetSolValueCalculatorIxData, SET_SOL_VALUE_CALC_IX_DISCM},
    },
    protocol_fee::set_protocol_fee_beneficiary::SET_PROTOCOL_FEE_BENEFICIARY_IX_DISCM,
    rebalance::set_rebal_auth::SET_REBAL_AUTH_IX_DISCM,
    swap::{exact_in::SWAP_EXACT_IN_IX_DISCM, exact_out::SWAP_EXACT_OUT_IX_DISCM, IxData},
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
    admin::{
        add_lst::process_add_lst,
        remove_lst::process_remove_lst,
        set_admin::{process_set_admin, set_admin_accs_checked},
        set_pricing_prog::{process_set_pricing_prog, set_pricing_prog_accs_checked},
        set_sol_value_calculator::process_set_sol_value_calculator,
    },
    protocol_fee::set_protocol_fee_beneficiary::{
        process_set_protocol_fee_beneficiary, set_protocol_fee_beneficiary_accs_checked,
    },
    rebalance::set_rebal_auth::{process_set_rebal_auth, set_rebal_auth_accs_checked},
    swap::{process_swap_exact_in, process_swap_exact_out},
    sync_sol_value::process_sync_sol_value,
};

mod instructions;
mod pricing;
mod svc;
mod utils;
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
        (&SWAP_EXACT_IN_IX_DISCM, data) => {
            sol_log("SwapExactIn");

            let args = IxData::<SWAP_EXACT_IN_IX_DISCM>::parse_no_discm(
                data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?,
            );

            process_swap_exact_in(abr, accounts, &args, cpi)
        }
        (&SWAP_EXACT_OUT_IX_DISCM, data) => {
            sol_log("SwapExactOut");

            let args = IxData::<SWAP_EXACT_OUT_IX_DISCM>::parse_no_discm(
                data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?,
            );

            process_swap_exact_out(abr, accounts, &args, cpi)
        }
        // admin ixs
        (&ADD_LST_IX_DISCM, _data) => {
            sol_log("AddLst");
            process_add_lst(abr, accounts, cpi)
        }
        (&REMOVE_LST_IX_DISCM, data) => {
            sol_log("RemoveLst");
            let lst_idx = RemoveLstIxData::parse_no_discm(
                data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?,
            ) as usize;
            process_remove_lst(abr, accounts, lst_idx, cpi)
        }
        (&SET_SOL_VALUE_CALC_IX_DISCM, data) => {
            sol_log("SetSolValueCalculator");
            let lst_idx = SetSolValueCalculatorIxData::parse_no_discm(
                data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?,
            ) as usize;
            process_set_sol_value_calculator(abr, accounts, lst_idx, cpi)
        }
        (&SET_ADMIN_IX_DISCM, _) => {
            sol_log("SetAdmin");
            let accs = set_admin_accs_checked(abr, accounts)?;
            process_set_admin(abr, accs)
        }
        (&SET_PRICING_PROG_IX_DISCM, _) => {
            sol_log("SetPricingProg");
            let accs = set_pricing_prog_accs_checked(abr, accounts)?;
            process_set_pricing_prog(abr, accs)
        }
        // protocol fee ixs
        (&SET_PROTOCOL_FEE_BENEFICIARY_IX_DISCM, _) => {
            sol_log("SetProtocolFeeBeneficiary");
            let accs = set_protocol_fee_beneficiary_accs_checked(abr, accounts)?;
            process_set_protocol_fee_beneficiary(abr, accs)
        }
        // rebalance ixs
        (&SET_REBAL_AUTH_IX_DISCM, _) => {
            sol_log("SetRebalAuth");
            let accs = set_rebal_auth_accs_checked(abr, accounts)?;
            process_set_rebal_auth(abr, accs)
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

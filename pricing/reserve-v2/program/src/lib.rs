use inf1_pp_reserve_v2_core::instructions::{
    admin::{
        remove_fee_entry::REMOVE_FEE_ENTRY_IX_DISCM,
        set_admin::SET_ADMIN_IX_DISCM,
        set_fee_entry::{SetFeeEntryIxData, SET_FEE_ENTRY_IX_DISCM},
    },
    init::INIT_IX_DISCM,
};
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;
use jiminy_cpi::account::{Abr, AccountHandle};
use jiminy_entrypoint::{
    program_entrypoint,
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA},
};
use jiminy_log::sol_log;

use crate::{
    instructions::{
        admin::{
            remove_fee_entry::{process_remove_fee_entry, remove_fee_entry_accs_checked},
            set_admin::{process_set_admin, set_admin_accs_checked},
            set_fee_entry::{process_set_fee_entry, set_fee_entry_accs_checked},
        },
        init::{init_accs_checked, process_init},
    },
    utils::ixdc,
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
        (&REMOVE_FEE_ENTRY_IX_DISCM, _data) => {
            sol_log("RemoveFeeEntry");
            let accs = remove_fee_entry_accs_checked(abr, accounts)?;
            process_remove_fee_entry(abr, accs)
        }
        (&SET_FEE_ENTRY_IX_DISCM, data) => {
            sol_log("SetFeeEntry");
            let accs = set_fee_entry_accs_checked(abr, accounts)?;
            let (threshold_nanos, fees) =
                SetFeeEntryIxData::parse_no_discm(ixdc(data)?).map_err(CustomProgErr)?;
            process_set_fee_entry(abr, accs, threshold_nanos, fees)
        }
        (&SET_ADMIN_IX_DISCM, _data) => {
            sol_log("SetAdmin");
            let accs = set_admin_accs_checked(abr, accounts)?;
            process_set_admin(abr, accs)
        }
        // Init
        (&INIT_IX_DISCM, _data) => {
            sol_log("Init");
            let accs = init_accs_checked(abr, accounts)?;
            process_init(abr, accs)
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

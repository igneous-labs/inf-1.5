// Re-exports
pub use inf1_svc_generic::{
    instructions::interface, keys::ConstAccs, pda::ConstPdas, traits::SolValCalc,
};
pub use jiminy_account::{Abr, AccountHandle};
pub use jiminy_cpi::{program_error::ProgramError, Cpi};
pub use program::*;

use inf1_svc_generic::instructions::interface::{
    lst_to_sol::LST_TO_SOL_IX_DISCM, sol_to_lst::SOL_TO_LST_IX_DISCM, IxData,
};

use jiminy_cpi::program_error::INVALID_INSTRUCTION_DATA;

use crate::utils::ix_data_as_arr;

mod instructions;
mod program;
mod utils;

#[inline]
pub fn process_ix(
    abr: &mut Abr,
    accs: &[AccountHandle<'_>],
    data: &[u8],
    prog: impl GenSvcProgram,
) -> Result<(), ProgramError> {
    let const_keys = prog.const_keys();
    let const_pdas = prog.const_pdas();

    match data.split_first().ok_or(INVALID_INSTRUCTION_DATA)? {
        // interface ixs
        (&LST_TO_SOL_IX_DISCM, data) | (&SOL_TO_LST_IX_DISCM, data) => {
            let amt = u64::from_le_bytes(*ix_data_as_arr(data)?);
            //let calc = prog.try_derive_calc(abr, accs, amt);
            todo!()
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

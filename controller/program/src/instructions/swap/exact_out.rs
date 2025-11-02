use inf1_ctl_jiminy::instructions::swap::IxArgs;
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::Cpi;

#[inline]
pub fn process_swap_exact_out(
    abr: &mut Abr,
    accounts: &[AccountHandle<'_>],
    args: &IxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    Ok(())
}

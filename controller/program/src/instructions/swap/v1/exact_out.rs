use inf1_ctl_jiminy::instructions::swap::IxArgs;
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::Cpi;

#[inline]
pub fn process_swap_exact_out(
    _abr: &mut Abr,
    _cpi: &mut Cpi,
    _accounts: &[AccountHandle<'_>],
    _args: &IxArgs,
) -> Result<(), ProgramError> {
    todo!()
}

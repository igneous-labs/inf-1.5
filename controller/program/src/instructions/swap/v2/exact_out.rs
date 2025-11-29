use inf1_ctl_jiminy::instructions::swap::IxArgs;
use jiminy_cpi::{account::Abr, program_error::ProgramError};
use jiminy_sysvar_clock::Clock;

use crate::instructions::swap::v2::{SwapV2IxAccounts, SwapV2Ty};

#[inline]
pub fn process_swap_exact_out_v2(
    _abr: &mut Abr,
    _accs: &SwapV2IxAccounts,
    _args: &IxArgs,
    _ty: SwapV2Ty,
    _clock: &Clock,
) -> Result<(), ProgramError> {
    todo!()
}

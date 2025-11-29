use inf1_ctl_jiminy::instructions::liquidity::remove::RemoveLiquidityIxArgs;
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
    Cpi,
};

#[inline]
pub fn process_remove_liquidity(
    _abr: &mut Abr,
    _cpi: &mut Cpi,
    _accounts: &[AccountHandle],
    _ix_args: RemoveLiquidityIxArgs,
) -> Result<(), ProgramError> {
    todo!()
}

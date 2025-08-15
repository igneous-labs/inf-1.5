use inf1_pp_core::{instructions::IxArgs, traits::main::PriceExactIn};
use inf1_pp_flatslab_core::errs::FlatSlabProgramErr;
use jiminy_entrypoint::program_error::ProgramError;
use jiminy_return_data::set_return_data;

use crate::{
    err::CustomProgErr,
    instructions::pricing::{
        common::{PricingIxPreAccHandles, PricingIxSufAccHandles},
        swap_pricing,
    },
    Accounts,
};

pub fn process_price_exact_in(
    accounts: &mut Accounts,
    pre: &PricingIxPreAccHandles,
    suf: &PricingIxSufAccHandles,
    args: IxArgs,
) -> Result<(), ProgramError> {
    let ret = swap_pricing(accounts, pre, suf)?
        .price_exact_in(args)
        .map_err(|e| CustomProgErr(FlatSlabProgramErr::Pricing(e)))?;
    set_return_data(&ret.to_le_bytes());
    Ok(())
}

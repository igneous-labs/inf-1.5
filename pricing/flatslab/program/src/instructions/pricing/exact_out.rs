use inf1_pp_core::{instructions::IxArgs, pair::Pair, traits::main::PriceExactOut};
use inf1_pp_flatslab_core::{accounts::Slab, errs::FlatSlabProgramErr};
use jiminy_entrypoint::program_error::{ProgramError, INVALID_ACCOUNT_DATA};
use jiminy_return_data::set_return_data;

use crate::{
    err::CustomProgErr,
    instructions::pricing::{
        common::{PriceIxPreAccHandles, PricingIxSufAccHandles},
        swap_pricing,
    },
    Accounts,
};

pub fn process_price_exact_out(
    accounts: &Accounts,
    pre: &PriceIxPreAccHandles,
    suf: &PricingIxSufAccHandles,
    args: IxArgs,
) -> Result<(), ProgramError> {
    let slab = Slab::of_acc_data(accounts.get(*suf.slab()).data()).ok_or(INVALID_ACCOUNT_DATA)?;
    let ret = swap_pricing(
        accounts,
        slab.entries(),
        Pair {
            inp: *pre.input_mint(),
            out: *pre.output_mint(),
        },
    )?
    .price_exact_out(args)
    .map_err(|e| CustomProgErr(FlatSlabProgramErr::Pricing(e)))?;
    set_return_data(&ret.to_le_bytes());
    Ok(())
}

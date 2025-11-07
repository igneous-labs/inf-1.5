use inf1_pp_core::{instructions::IxArgs, pair::Pair, traits::main::PriceExactIn};
use inf1_pp_flatslab_core::{accounts::Slab, errs::FlatSlabProgramErr};
use jiminy_cpi::account::Abr;
use jiminy_entrypoint::program_error::{ProgramError, INVALID_ACCOUNT_DATA};
use jiminy_return_data::set_return_data;

use crate::{
    err::CustomProgErr,
    instructions::pricing::{common::PriceIxAccHandles, swap_pricing},
};

pub fn process_price_exact_in(
    abr: &Abr,
    PriceIxAccHandles { ix_prefix, suf }: &PriceIxAccHandles,
    args: IxArgs,
) -> Result<(), ProgramError> {
    let slab = Slab::of_acc_data(abr.get(*suf.slab()).data()).ok_or(INVALID_ACCOUNT_DATA)?;
    let ret = swap_pricing(
        abr,
        slab.entries(),
        Pair {
            inp: *ix_prefix.input_mint(),
            out: *ix_prefix.output_mint(),
        },
    )?
    .price_exact_in(args)
    .map_err(FlatSlabProgramErr::Pricing)
    .map_err(CustomProgErr)?;
    set_return_data(&ret.to_le_bytes());
    Ok(())
}

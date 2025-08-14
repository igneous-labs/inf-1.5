use inf1_pp_core::{instructions::IxArgs, traits::main::PriceExactIn};
use inf1_pp_flatslab_core::{accounts::Slab, errs::FlatSlabProgramErr, pricing::FlatSlabPricing};
use jiminy_entrypoint::program_error::{BuiltInProgramError, ProgramError};
use jiminy_return_data::set_return_data;

use crate::{
    err::CustomProgErr,
    instructions::pricing::common::{PricingIxPreAccHandles, PricingIxSufAccHandles},
    Accounts,
};

pub fn process_price_exact_in(
    accounts: &mut Accounts,
    pre: &PricingIxPreAccHandles,
    suf: &PricingIxSufAccHandles,
    args: IxArgs,
) -> Result<(), ProgramError> {
    let slab = Slab::of_acc_data(accounts.get(*suf.slab()).data()).ok_or(
        ProgramError::from_builtin(BuiltInProgramError::InvalidAccountData),
    )?;
    let entries = slab.entries();
    let [inp, out] = [pre.input_mint(), pre.output_mint()].map(|mint_handle| {
        entries
            .find_by_mint(accounts.get(*mint_handle).key())
            .map_err(|e| CustomProgErr(FlatSlabProgramErr::MintNotFound(e)))
    });
    let inp = inp?;
    let out = out?;

    let ret = FlatSlabPricing {
        inp_fee_nanos: inp.inp_fee_nanos(),
        out_fee_nanos: out.out_fee_nanos(),
    }
    .price_exact_in(args)
    .map_err(|e| CustomProgErr(FlatSlabProgramErr::Pricing(e)))?;

    set_return_data(&ret.to_le_bytes());

    Ok(())
}

use inf1_pp_core::{instructions::IxArgs, pair::Pair, traits::main::PriceExactIn};
use inf1_pp_flatslab_core::{accounts::Slab, errs::FlatSlabProgramErr, keys::LP_MINT_ID};
use jiminy_entrypoint::program_error::{ProgramError, INVALID_ACCOUNT_DATA};
use jiminy_return_data::set_return_data;

use crate::{
    err::CustomProgErr,
    instructions::pricing::common::{LpIxPreAccHandles, PricingIxSufAccHandles},
    Accounts,
};

#[allow(deprecated)]
pub fn process_price_lp_tokens_to_redeem(
    accounts: &mut Accounts,
    pre: &LpIxPreAccHandles,
    suf: &PricingIxSufAccHandles,
    args: IxArgs,
) -> Result<(), ProgramError> {
    let slab = Slab::of_acc_data(accounts.get(*suf.slab()).data()).ok_or(INVALID_ACCOUNT_DATA)?;
    let pair = Pair {
        inp: &LP_MINT_ID,
        out: accounts.get(*pre.mint()).key(),
    };
    let ret = slab
        .entries()
        .pricing(&pair)
        .map_err(|e| CustomProgErr(FlatSlabProgramErr::MintNotFound(e)))?
        .price_exact_in(args)
        .map_err(|e| CustomProgErr(FlatSlabProgramErr::Pricing(e)))?;
    set_return_data(&ret.to_le_bytes());
    Ok(())
}

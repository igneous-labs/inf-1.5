use inf1_pp_core::{instructions::IxArgs, pair::Pair, traits::main::PriceExactOut};
use inf1_pp_reserve_v2_core::{
    errs::ReserveV2ProgramErr,
    pricing::FlatPricing,
    route::{classify_route, RouteKind},
};
use inf1_pp_reserve_v2_jiminy::{account_utils::pricing_state_checked, program_err::CustomProgErr};
use jiminy_cpi::{account::Abr, program_error::ProgramError};
use jiminy_return_data::set_return_data;

use crate::instructions::pricing::{range_out_pricing, PriceIxAccHandles};

pub fn process_price_exact_out(
    abr: &Abr,
    accs: &PriceIxAccHandles,
    args: IxArgs,
) -> Result<(), ProgramError> {
    let mints = Pair {
        inp: *accs.ix_prefix.input_mint(),
        out: *accs.ix_prefix.output_mint(),
    }
    .map(|handle| abr.get(handle).key());
    let route = classify_route(mints.inp, mints.out).map_err(CustomProgErr)?;

    let (_, entries) = pricing_state_checked(abr.get(*accs.suf.pricing_state()))?;
    let Pair {
        inp: input_entry,
        out: output_entry,
    } = mints.try_map(|mint| {
        entries
            .find_by_mint(mint)
            .map_err(|e| CustomProgErr(ReserveV2ProgramErr::MintNotFound(e)))
    })?;

    let ret = match route {
        RouteKind::Flat => {
            FlatPricing::from_entries(input_entry, output_entry).price_exact_out(args)
        }
        RouteKind::RangeOut => {
            range_out_pricing(abr, &accs.suf, input_entry, output_entry)?.price_exact_out(args)
        }
    }
    .map_err(CustomProgErr)?;
    set_return_data(&ret.to_le_bytes());
    Ok(())
}

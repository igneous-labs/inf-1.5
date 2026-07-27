use inf1_pp_core::{instructions::IxArgs, traits::main::PriceExactIn};
use inf1_pp_reserve_v2_core::{pricing::FlatPricing, route::RouteKind};
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;
use jiminy_cpi::{account::Abr, program_error::ProgramError};
use jiminy_return_data::set_return_data;

use crate::instructions::pricing::{range_out_pricing, route_and_fee_entries, PriceIxAccHandles};

pub fn process_price_exact_in(
    abr: &Abr,
    accs: &PriceIxAccHandles,
    args: IxArgs,
) -> Result<(), ProgramError> {
    let (route, input_entry, output_entry) = route_and_fee_entries(abr, accs)?;

    let ret = match route {
        RouteKind::Flat => {
            FlatPricing::from_entries(input_entry, output_entry).price_exact_in(args)
        }
        RouteKind::RangeOut => {
            range_out_pricing(abr, &accs.suf, input_entry, output_entry)?.price_exact_in(args)
        }
    }
    .map_err(CustomProgErr)?;
    set_return_data(&ret.to_le_bytes());
    Ok(())
}

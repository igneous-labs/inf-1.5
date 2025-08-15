use inf1_pp_core::instructions::{
    price::{exact_in::PRICE_EXACT_IN_IX_DISCM, exact_out::PRICE_EXACT_OUT_IX_DISCM},
    IxArgs,
};
use jiminy_entrypoint::{
    program_entrypoint,
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA},
};

use crate::instructions::pricing::{
    lp_accs_checked, pricing_accs_checked, process_price_exact_in, process_price_exact_out,
    process_price_lp_tokens_to_mint, process_price_lp_tokens_to_redeem,
};

#[allow(deprecated)]
use inf1_pp_core::instructions::deprecated::lp::{
    mint::PRICE_LP_TOKENS_TO_MINT_IX_DISCM, redeem::PRICE_LP_TOKENS_TO_REDEEM_IX_DISCM,
};

mod err;
mod instructions;
mod utils;

// Re-exports for integration tests
pub use err::*;

const MAX_ACCS: usize = 4;

type Accounts<'account> = jiminy_entrypoint::account::Accounts<'account, MAX_ACCS>;

program_entrypoint!(process_ix, MAX_ACCS);

fn process_ix(
    accounts: &mut Accounts,
    data: &[u8],
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    match data.split_first().ok_or(INVALID_INSTRUCTION_DATA)? {
        // interface ixs
        (&PRICE_EXACT_IN_IX_DISCM, data) => {
            let (pre, suf) = pricing_accs_checked(accounts)?;
            let args = IxArgs::parse(data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?);
            process_price_exact_in(accounts, &pre, &suf, args)
        }
        (&PRICE_EXACT_OUT_IX_DISCM, data) => {
            let (pre, suf) = pricing_accs_checked(accounts)?;
            let args = IxArgs::parse(data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?);
            process_price_exact_out(accounts, &pre, &suf, args)
        }
        #[allow(deprecated)]
        (&PRICE_LP_TOKENS_TO_MINT_IX_DISCM, data) => {
            let (pre, suf) = lp_accs_checked(accounts)?;
            let args = IxArgs::parse(data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?);
            process_price_lp_tokens_to_mint(accounts, &pre, &suf, args)
        }
        #[allow(deprecated)]
        (&PRICE_LP_TOKENS_TO_REDEEM_IX_DISCM, data) => {
            let (pre, suf) = lp_accs_checked(accounts)?;
            let args = IxArgs::parse(data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?);
            process_price_lp_tokens_to_redeem(accounts, &pre, &suf, args)
        }

        // admin ixs
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

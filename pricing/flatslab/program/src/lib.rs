use inf1_pp_core::instructions::{
    price::{exact_in::PRICE_EXACT_IN_IX_DISCM, exact_out::PRICE_EXACT_OUT_IX_DISCM},
    IxArgs,
};
use inf1_pp_flatslab_core::instructions::{
    admin::{
        set_admin::SET_ADMIN_IX_DISCM,
        set_lst_fee::{SetLstFeeIxArgs, SET_LST_FEE_IX_DISCM},
    },
    init::INIT_IX_DISCM,
};
use jiminy_entrypoint::{
    program_entrypoint,
    program_error::{ProgramError, INVALID_INSTRUCTION_DATA},
};

use crate::instructions::{
    admin::{
        process_set_admin, process_set_lst_fee, set_admin_accs_checked, set_lst_fee_accs_checked,
    },
    init::{init_accs_checked, process_init},
    pricing::{
        lp_accs_checked, pricing_accs_checked, process_price_exact_in, process_price_exact_out,
        process_price_lp_tokens_to_mint, process_price_lp_tokens_to_redeem,
    },
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
pub use utils::*;

/// Max possible accounts is 5 (SetLstFee)
const MAX_ACCS: usize = 5;

type Accounts<'account> = jiminy_entrypoint::account::Accounts<'account, MAX_ACCS>;

program_entrypoint!(process_ix, MAX_ACCS);

fn process_ix(
    accounts: &mut Accounts,
    data: &[u8],
    prog_id: &[u8; 32],
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

        // init
        (&INIT_IX_DISCM, _data) => {
            let accs = init_accs_checked(accounts)?;
            process_init(accounts, accs, prog_id)
        }

        // admin ixs
        (&SET_ADMIN_IX_DISCM, _data) => {
            let accs = set_admin_accs_checked(accounts)?;
            process_set_admin(accounts, accs)
        }
        (&SET_LST_FEE_IX_DISCM, data) => {
            let accs = set_lst_fee_accs_checked(accounts)?;
            let args =
                SetLstFeeIxArgs::parse(data.try_into().map_err(|_e| INVALID_INSTRUCTION_DATA)?);
            process_set_lst_fee(accounts, accs, args)
        }

        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

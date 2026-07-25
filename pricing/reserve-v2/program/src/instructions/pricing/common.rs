use inf1_ctl_jiminy::account_utils::pool_state_v2_checked;
use inf1_pp_core::instructions::price::{IxAccs, IxPreAccs};
use inf1_pp_reserve_v2_core::{
    errs::{ReserveV2ProgramErr, WsolBalanceGtPoolSolValueErr},
    instructions::pricing::{IxSufAccs, IxSufAccsDestr, IxSufKeys},
    pda::CONST_PDA_KEYS_OWNED,
    pricing::RangeOutPricing,
    typedefs::FeeEntry,
};
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use sanctum_spl_token_core::state::account::{RawTokenAccount, TokenAccount};

use crate::utils::{asfc, verify_pks};

pub type PriceIxAccHandles<'a> = IxAccs<AccountHandle<'a>, IxSufAccs<AccountHandle<'a>>>;

pub fn pricing_accs_checked<'acc>(
    abr: &Abr,
    accounts: &[AccountHandle<'acc>],
) -> Result<PriceIxAccHandles<'acc>, ProgramError> {
    let (pre, rest) = asfc(accounts)?;
    let (suf, _) = asfc(rest)?;

    let ix_prefix = IxPreAccs(*pre);
    let suf = IxSufAccs(*suf);

    verify_pks(
        abr,
        &suf.0,
        &IxSufKeys::const_from_destr(IxSufAccsDestr {
            pricing_state: CONST_PDA_KEYS_OWNED.pricing_state(),
            pool_state: CONST_PDA_KEYS_OWNED.pool_state(),
            wsol_reserves: CONST_PDA_KEYS_OWNED.wsol_reserves(),
        })
        .0,
    )?;

    Ok(IxAccs::new(ix_prefix, suf))
}

pub fn range_out_pricing(
    abr: &Abr,
    suf: &IxSufAccs<AccountHandle<'_>>,
    input_entry: &FeeEntry,
    output_entry: &FeeEntry,
) -> Result<RangeOutPricing, ProgramError> {
    let pool_sol_value = pool_state_v2_checked(abr.get(*suf.pool_state()))?.total_sol_value;
    if pool_sol_value == 0 {
        return Err(CustomProgErr(ReserveV2ProgramErr::ZeroPoolSolValue).into());
    }

    let wsol_reserves_acc = abr.get(*suf.wsol_reserves());
    let wsol_balance = RawTokenAccount::of_acc_data(wsol_reserves_acc.data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(ProgramError::from(INVALID_ACCOUNT_DATA))?;
    if wsol_balance > pool_sol_value {
        return Err(
            CustomProgErr(ReserveV2ProgramErr::WsolBalanceGtPoolSolValue(
                WsolBalanceGtPoolSolValueErr {
                    pool_sol_value,
                    wsol_balance,
                },
            ))
            .into(),
        );
    }

    Ok(RangeOutPricing::from_entries(
        input_entry,
        output_entry,
        pool_sol_value,
        wsol_balance,
    ))
}

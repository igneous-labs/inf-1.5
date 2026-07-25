use inf1_ctl_jiminy::account_utils::pool_state_v2_checked;
use inf1_pp_core::{
    instructions::price::{IxAccs, IxPreAccs},
    pair::Pair,
};
use inf1_pp_reserve_v2_core::{
    errs::{ReserveV2ProgramErr, WsolBalanceGtPoolSolValueErr},
    instructions::pricing::{IxSufAccs, ReserveV2PpAccs},
    pricing::RangeOutPricing,
    route::{classify_route, RouteKind},
    typedefs::FeeEntry,
};
use inf1_pp_reserve_v2_jiminy::{account_utils::pricing_state_checked, program_err::CustomProgErr};
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

    let expected = ReserveV2PpAccs::MAINNET.pp_suf_keys_owned();
    verify_pks(abr, &suf.0, &expected.0.each_ref())?;

    Ok(IxAccs::new(ix_prefix, suf))
}

pub fn route_and_fee_entries<'a>(
    abr: &'a Abr,
    accs: &PriceIxAccHandles<'_>,
) -> Result<(RouteKind, &'a FeeEntry, &'a FeeEntry), ProgramError> {
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
            .find_idx_by_mint(mint)
            .map(|i| &entries.0[i])
            .map_err(|e| CustomProgErr(ReserveV2ProgramErr::MintNotFound(e)))
    })?;

    Ok((route, input_entry, output_entry))
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

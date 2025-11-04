use inf1_core::quote::swap::{exact_in::quote_exact_in, SwapQuoteArgs};
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    cpi::{LstToSolRetVal, PricingRetVal, SolToLstRetVal},
    err::Inf1CtlErr,
    instructions::swap::IxArgs,
    program_err::Inf1CtlCustomProgErr,
};
use inf1_jiminy::SwapQuoteProgErr;
use inf1_pp_jiminy::{
    cpi::price::lp::cpi_price_exact_in, instructions::price::exact_in::PriceExactInIxArgs,
};
use inf1_svc_jiminy::cpi::{cpi_lst_to_sol, cpi_sol_to_lst};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::{
    RawTokenAccount, TokenAccount,
};

use crate::{
    instructions::swap::internal_utils::{
        swap_checked, sync_inp_out_sol_vals, transfer_swap_tokens, SwapIxAccounts,
    },
    pricing::{NewPpIxPreAccsBuilder, PriceExactInIxAccountHandles},
    svc::{NewSvcIxPreAccsBuilder, SvcIxAccountHandles},
    Cpi,
};

#[inline]
pub fn process_swap_exact_in(
    abr: &mut Abr,
    accounts: &[AccountHandle<'_>],
    args: &IxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let swap_accs = swap_checked(abr, accounts, args)?;

    let SwapIxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        pricing_prog,
        pricing,
    } = swap_accs;

    sync_inp_out_sol_vals(abr, cpi, args, &swap_accs)?;

    let inp_retval = cpi_lst_to_sol(
        cpi,
        abr,
        inp_calc_prog,
        args.amount,
        SvcIxAccountHandles::new(
            NewSvcIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.inp_lst_mint())
                .build(),
            inp_calc,
        ),
    )?;

    let inp_sol_value = *inp_retval.start();
    if inp_sol_value == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    let out_sol_val = cpi_price_exact_in(
        cpi,
        abr,
        pricing_prog,
        PriceExactInIxArgs {
            amt: args.amount,
            sol_value: inp_sol_value,
        },
        PriceExactInIxAccountHandles::new(
            NewPpIxPreAccsBuilder::start()
                .with_input_mint(*ix_prefix.inp_lst_mint())
                .with_output_mint(*ix_prefix.out_lst_mint())
                .build(),
            pricing,
        ),
    )?;

    let out_retval = cpi_sol_to_lst(
        cpi,
        abr,
        out_calc_prog,
        out_sol_val,
        SvcIxAccountHandles::new(
            NewSvcIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.out_lst_mint())
                .build(),
            out_calc,
        ),
    )?;

    if *out_retval.start() < args.limit {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let inp_calc_retval = LstToSolRetVal(inp_retval);
    let out_calc_retval = SolToLstRetVal(out_retval);
    let pricing_retval = PricingRetVal(out_sol_val);

    let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let pool_trading_protocol_fee_bps = pool.trading_protocol_fee_bps;
    let start_total_sol_value = pool.total_sol_value;

    let out_lst_balance =
        RawTokenAccount::of_acc_data(abr.get(*ix_prefix.out_pool_reserves()).data())
            .and_then(TokenAccount::try_from_raw)
            .map(|a| a.amount())
            .ok_or(INVALID_ACCOUNT_DATA)?;

    let quote = quote_exact_in(SwapQuoteArgs {
        amt: args.amount,
        out_reserves: out_lst_balance,
        trading_protocol_fee_bps: pool_trading_protocol_fee_bps,
        inp_calc: inp_calc_retval,
        out_calc: out_calc_retval,
        pricing: pricing_retval,
        inp_mint: *abr.get(*ix_prefix.inp_lst_mint()).key(),
        out_mint: *abr.get(*ix_prefix.out_lst_mint()).key(),
    })
    .map_err(|e| ProgramError::from(SwapQuoteProgErr(e)))?;

    transfer_swap_tokens(abr, cpi, &quote, &ix_prefix)?;

    sync_inp_out_sol_vals(abr, cpi, args, &swap_accs)?;

    let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let final_total_sol_value = pool.total_sol_value;

    if final_total_sol_value < start_total_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    Ok(())
}

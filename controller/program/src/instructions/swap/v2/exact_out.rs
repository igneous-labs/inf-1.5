use inf1_core::quote::swap::{exact_out::quote_exact_out, QuoteArgs};
use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    cpi::{PricingRetVal, SolValCalcRetVal},
    err::Inf1CtlErr,
    instructions::swap::IxArgs,
    program_err::Inf1CtlCustomProgErr,
    svc::InfCalc,
    typedefs::pool_sv::PoolSvLamports,
};
use inf1_pp_jiminy::{
    cpi::price::swap::{cpi_price_exact_out, PriceExactOutIxAccountHandles},
    instructions::price::{
        exact_out::PriceExactOutIxArgs, NewIxPreAccsBuilder as NewPpIxPreAccsBuilder,
    },
};
use inf1_svc_jiminy::{
    cpi::{cpi_lst_to_sol, cpi_sol_to_lst, IxAccountHandles as SvcIxAccountHandles},
    instructions::NewIxPreAccsBuilder as NewSvcIxPreAccsBuilder,
};
use jiminy_cpi::{account::Abr, program_error::ProgramError};
use jiminy_sysvar_clock::Clock;

use crate::{
    err::quote_err_to_inf1_ctl_err,
    instructions::swap::v2::{
        final_sync, final_sync_aux_post_movement, final_sync_aux_pre_movement, initial_sync,
        move_tokens, SwapCpiRetVals, SwapV2CtlIxAccounts, SwapV2IxAccounts,
    },
    token::{checked_mint_of, get_token_account_amount},
    Cpi,
};

#[inline]
pub fn process_swap_exact_out_v2(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &SwapV2CtlIxAccounts,
    args: &IxArgs,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.as_ref().ix_prefix.pool_state()))?;
    pool.release_yield(clock.slot)
        .map_err(Inf1CtlCustomProgErr)?;

    initial_sync(abr, cpi, accs, args)?;

    let SwapCpiRetVals {
        inp_calc,
        out_calc,
        pricing,
    } = exec_calc_cpis_unchecked(abr, cpi, accs, args.amount)?;

    let out_reserves = match accs {
        SwapV2CtlIxAccounts::AddLiq(_) => u64::MAX,
        SwapV2CtlIxAccounts::RemLiq(_) | SwapV2CtlIxAccounts::Swap(_) => {
            get_token_account_amount(abr.get(*accs.as_ref().ix_prefix.out_pool_reserves()))?
        }
    };

    let quote = quote_exact_out(&QuoteArgs {
        amt: args.amount,
        out_reserves,
        inp_calc,
        out_calc,
        pricing,
        inp_mint: *abr.get(*accs.as_ref().ix_prefix.inp_mint()).key(),
        out_mint: *abr.get(*accs.as_ref().ix_prefix.out_mint()).key(),
    })
    .map_err(quote_err_to_inf1_ctl_err)
    .map_err(Inf1CtlCustomProgErr)?;

    if quote.inp > args.limit {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let aux_pre = final_sync_aux_pre_movement(abr, accs)?;

    move_tokens(abr, cpi, accs, &quote)?;

    let aux = final_sync_aux_post_movement(abr, accs.as_ref().ix_prefix, aux_pre)?;

    final_sync(abr, cpi, accs.as_ref(), args, &aux)?;

    Ok(())
}

/// "unchecked" because it does not assert anything about the values;
/// rely on [`quote_exact_out`] for those checks
#[inline]
fn exec_calc_cpis_unchecked(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &SwapV2CtlIxAccounts,
    amount: u64,
) -> Result<SwapCpiRetVals, ProgramError> {
    let SwapV2IxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        pricing_prog,
        pricing,
    } = accs.as_ref();
    let pool_lamports = PoolSvLamports::from_pool_state_v2(pool_state_v2_checked(
        abr.get(*ix_prefix.pool_state()),
    )?);

    let out_retval = match accs {
        SwapV2CtlIxAccounts::AddLiq(_) => {
            let inf_supply = checked_mint_of(abr.get(*ix_prefix.out_mint()))?.supply();
            InfCalc {
                pool_lamports,
                mint_supply: inf_supply,
            }
            .svc_lst_to_sol(amount)
            .map_err(Inf1CtlErr::from)
            .map_err(Inf1CtlCustomProgErr::from)?
        }
        SwapV2CtlIxAccounts::Swap(_) | SwapV2CtlIxAccounts::RemLiq(_) => cpi_lst_to_sol(
            cpi,
            abr,
            out_calc_prog,
            amount,
            SvcIxAccountHandles {
                ix_prefix: NewSvcIxPreAccsBuilder::start()
                    .with_lst_mint(*ix_prefix.out_mint())
                    .build(),
                suf: out_calc,
            },
        )?,
    };

    let out_sol_value = *out_retval.end();
    let inp_sol_val = cpi_price_exact_out(
        cpi,
        abr,
        pricing_prog,
        PriceExactOutIxArgs {
            amt: amount,
            sol_value: out_sol_value,
        },
        &PriceExactOutIxAccountHandles {
            ix_prefix: NewPpIxPreAccsBuilder::start()
                .with_input_mint(*ix_prefix.inp_mint())
                .with_output_mint(*ix_prefix.out_mint())
                .build(),
            suf: pricing,
        },
    )?;

    let inp_retval = match accs {
        SwapV2CtlIxAccounts::RemLiq(_) => {
            let inf_supply = checked_mint_of(abr.get(*ix_prefix.inp_mint()))?.supply();
            InfCalc {
                pool_lamports,
                mint_supply: inf_supply,
            }
            .svc_sol_to_lst(inp_sol_val)
            .map_err(Inf1CtlErr::from)
            .map_err(Inf1CtlCustomProgErr)?
        }
        SwapV2CtlIxAccounts::Swap(_) | SwapV2CtlIxAccounts::AddLiq(_) => cpi_sol_to_lst(
            cpi,
            abr,
            inp_calc_prog,
            inp_sol_val,
            SvcIxAccountHandles {
                ix_prefix: NewSvcIxPreAccsBuilder::start()
                    .with_lst_mint(*ix_prefix.inp_mint())
                    .build(),
                suf: inp_calc,
            },
        )?,
    };

    Ok(SwapCpiRetVals {
        out_calc: SolValCalcRetVal(out_retval),
        pricing: PricingRetVal(inp_sol_val),
        inp_calc: SolValCalcRetVal(inp_retval),
    })
}

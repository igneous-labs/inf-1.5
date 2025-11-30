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
        final_sync, final_sync_aux_post_changes, final_sync_aux_pre_changes, initial_sync,
        move_tokens, SwapCpiRetVals, SwapV2IxAccounts, SwapV2Ty,
    },
    token::{checked_mint_of, get_token_account_amount},
    yield_release::release_yield,
    Cpi,
};

#[inline]
pub fn process_swap_exact_out_v2(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &SwapV2IxAccounts,
    args: &IxArgs,
    ty: SwapV2Ty,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.ix_prefix.pool_state()))?;
    release_yield(pool, clock)?;

    initial_sync(abr, cpi, accs, args, ty)?;

    let SwapCpiRetVals {
        inp_calc,
        out_calc,
        pricing,
    } = exec_calc_cpis_unchecked(abr, cpi, accs, args.amount, ty)?;

    let out_reserves = match ty {
        SwapV2Ty::AddLiq(_) => u64::MAX,
        SwapV2Ty::RemLiq(_) | SwapV2Ty::Swap(_) => {
            get_token_account_amount(abr.get(*accs.ix_prefix.out_pool_reserves()))?
        }
    };

    let quote = quote_exact_out(&QuoteArgs {
        amt: args.amount,
        out_reserves,
        inp_calc,
        out_calc,
        pricing,
        inp_mint: *abr.get(*accs.ix_prefix.inp_mint()).key(),
        out_mint: *abr.get(*accs.ix_prefix.out_mint()).key(),
    })
    .map_err(quote_err_to_inf1_ctl_err)
    .map_err(Inf1CtlCustomProgErr)?;

    if quote.inp > args.limit {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let aux_pre = final_sync_aux_pre_changes(abr, accs.ix_prefix, ty)?;

    move_tokens(abr, cpi, accs.ix_prefix, &quote, ty)?;

    let aux = final_sync_aux_post_changes(abr, accs.ix_prefix, aux_pre)?;

    final_sync(abr, cpi, accs, args, &aux)?;

    Ok(())
}

/// "unchecked" because it does not assert anything about the values;
/// rely on [`quote_exact_out`] for those checks
#[inline]
fn exec_calc_cpis_unchecked(
    abr: &mut Abr,
    cpi: &mut Cpi,
    SwapV2IxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        pricing_prog,
        pricing,
    }: &SwapV2IxAccounts,
    amount: u64,
    ty: SwapV2Ty,
) -> Result<SwapCpiRetVals, ProgramError> {
    let pool_lamports = PoolSvLamports::from_pool_state_v2(pool_state_v2_checked(
        abr.get(*ix_prefix.pool_state()),
    )?);

    let out_retval = match ty {
        SwapV2Ty::AddLiq(_) => {
            let inf_supply = checked_mint_of(abr.get(*ix_prefix.out_mint()))?.supply();
            InfCalc {
                pool_lamports,
                mint_supply: inf_supply,
            }
            .svc_lst_to_sol(amount)
            .map_err(Inf1CtlErr::from)
            .map_err(Inf1CtlCustomProgErr::from)?
        }
        SwapV2Ty::Swap(_) | SwapV2Ty::RemLiq(_) => cpi_lst_to_sol(
            cpi,
            abr,
            &out_calc_prog.unwrap(), // panic if we messed up setting up according to ty
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
        &pricing_prog.unwrap(), // panic if we messed up setting up according to ty
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

    let inp_retval = match ty {
        SwapV2Ty::RemLiq(_) => {
            let inf_supply = checked_mint_of(abr.get(*ix_prefix.inp_mint()))?.supply();
            InfCalc {
                pool_lamports,
                mint_supply: inf_supply,
            }
            .svc_sol_to_lst(amount)
            .map_err(Inf1CtlErr::from)
            .map_err(Inf1CtlCustomProgErr::from)?
        }
        SwapV2Ty::Swap(_) | SwapV2Ty::AddLiq(_) => cpi_sol_to_lst(
            cpi,
            abr,
            &inp_calc_prog.unwrap(), // panic if we messed up setting up according to ty
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

use crate::svc::lst_sync_sol_val_unchecked;
use inf1_core::instructions::sync_sol_value::SyncSolValueIxAccs;
#[allow(deprecated)]
use inf1_core::{
    instructions::liquidity::remove::RemoveLiquidityIxAccs,
    quote::liquidity::remove::{quote_remove_liq, RemoveLiqQuoteArgs},
};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    cpi::{PricingRetVal, RemoveLiquidityPreAccountHandles, SolToLstRetVal},
    err::Inf1CtlErr,
    instructions::{
        liquidity::{
            remove::{NewRemoveLiquidityIxPreAccsBuilder, RemoveLiquidityIxArgs},
            IxPreAccs,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::{create_raw_pool_reserves_addr, create_raw_protocol_fee_accumulator_addr},
    program_err::Inf1CtlCustomProgErr,
    seeds::POOL_SEED_SIGNER,
};
use inf1_jiminy::RemoveLiqQuoteProgErr;

#[allow(deprecated)]
use inf1_pp_core::instructions::deprecated::lp::redeem::PriceLpTokensToRedeemIxArgs;
use inf1_pp_jiminy::cpi::deprecated::lp::{
    cpi_price_lp_tokens_to_redeem, PriceLpTokensToRedeemIxAccountHandles,
};

use inf1_svc_jiminy::{
    cpi::{cpi_sol_to_lst, IxAccountHandles as SvcIxAccountHandles},
    instructions::NewIxPreAccsBuilder as NewSvcIxPreAccsBuilder,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
    Cpi,
};

#[allow(deprecated)]
use crate::pricing::DeprecatedNewPpIxPreAccsBuilder;
use sanctum_spl_token_jiminy::{
    instructions::transfer::transfer_checked_ix_account_handle_perms,
    sanctum_spl_token_core::{
        instructions::{
            burn::{BurnIxData, NewBurnIxAccsBuilder},
            transfer::{NewTransferCheckedIxAccsBuilder, TransferCheckedIxData},
        },
        state::{
            account::{RawTokenAccount, TokenAccount},
            mint::{Mint, RawMint},
        },
    },
};

use crate::verify::{verify_not_rebalancing_and_not_disabled, verify_pks};

#[allow(deprecated)]
pub type RemoveLiquidityIxAccounts<'a, 'acc> = RemoveLiquidityIxAccs<
    AccountHandle<'acc>,
    RemoveLiquidityPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
>;

/// Returns an `RemoveLiquidityIxAccs` struct containing the instruction prefix and all required suffix accounts (for sol value calc program and pricing program).
#[inline]
fn remove_liquidity_accs_checked<'a, 'acc>(
    abr: &Abr,
    accounts: &'a [AccountHandle<'acc>],
    ix_args: RemoveLiquidityIxArgs,
) -> Result<RemoveLiquidityIxAccounts<'a, 'acc>, ProgramError> {
    if ix_args.amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let ix_prefix = IxPreAccs(*ix_prefix);
    let list = LstStatePackedList::of_acc_data(abr.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let lst_state = list
        .0
        .get(ix_args.lst_index as usize)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let lst_mint_acc = abr.get(*ix_prefix.lst_mint());
    let token_prog = lst_mint_acc.owner();
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state() };

    let expected_reserves =
        create_raw_pool_reserves_addr(token_prog, &lst_state.mint, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let expected_protocol_fee_accumulator = create_raw_protocol_fee_accumulator_addr(
        token_prog,
        &lst_state.mint,
        &lst_state.protocol_fee_accumulator_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let expected_pks = NewRemoveLiquidityIxPreAccsBuilder::start()
        .with_lst_mint(&lst_state.mint)
        .with_lp_token_mint(&pool.lp_token_mint)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_lst_token_program(abr.get(*(ix_prefix.lst_mint())).owner())
        .with_lp_token_program(abr.get(*(ix_prefix.lp_token_mint())).owner())
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_reserves(&expected_reserves)
        // These can be arbitrary accs bc they belong to the user adding liquidity to INF
        .with_signer(abr.get(*ix_prefix.signer()).key())
        .with_lst_acc(abr.get(*ix_prefix.lst_acc()).key())
        .with_lp_acc(abr.get(*ix_prefix.lp_acc()).key())
        .build();

    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    let (lst_calc_all, pricing_all) = suf
        .split_at_checked((ix_args.lst_value_calc_accs).into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let [Some((lst_calc_prog, lst_calc_acc)), Some((pricing_prog, pricing_accs))] =
        [lst_calc_all, pricing_all].map(|arr| arr.split_first())
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    verify_pks(
        abr,
        &[*lst_calc_prog, *pricing_prog],
        &[&lst_state.sol_value_calculator, &pool.pricing_program],
    )?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    Ok(RemoveLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog: *lst_calc_prog,
        lst_calc: lst_calc_acc,
        pricing_prog: *pricing_prog,
        pricing: pricing_accs,
    })
}

#[inline]
#[allow(deprecated)]
pub fn process_remove_liquidity(
    abr: &mut Abr,
    accounts: &[AccountHandle],
    ix_args: RemoveLiquidityIxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let RemoveLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog,
        lst_calc,
        pricing_prog,
        pricing,
    } = remove_liquidity_accs_checked(abr, accounts, ix_args)?;

    let sync_sol_val_calcs = SyncSolValueIxAccs {
        ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.lst_mint())
            .with_pool_state(*ix_prefix.pool_state())
            .with_lst_state_list(*ix_prefix.lst_state_list())
            .with_pool_reserves(*ix_prefix.pool_reserves())
            .build(),
        calc_prog: lst_calc_prog,
        calc: lst_calc,
    };

    lst_sync_sol_val_unchecked(abr, cpi, sync_sol_val_calcs, ix_args.lst_index as usize)?;

    let pool_sol_value = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?
        .total_sol_value;

    let lp_token_supply = RawMint::of_acc_data(abr.get(*ix_prefix.lp_token_mint()).data())
        .and_then(Mint::try_from_raw)
        .map(|a| a.supply())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let lp_tokens_sol_value = ix_args
        .amount
        .checked_mul(pool_sol_value)
        .unwrap()
        .checked_div(lp_token_supply)
        .unwrap();

    let lp_tokens_sol_value_after_fees = PricingRetVal(cpi_price_lp_tokens_to_redeem(
        cpi,
        abr,
        pricing_prog,
        PriceLpTokensToRedeemIxArgs {
            sol_value: lp_tokens_sol_value,
            amt: ix_args.amount,
        },
        PriceLpTokensToRedeemIxAccountHandles::new(
            DeprecatedNewPpIxPreAccsBuilder::start()
                // Mint of the output lst
                .with_mint(*ix_prefix.lst_mint())
                .build(),
            pricing,
        ),
    )?);

    // Value of lp tokens in terms of sol regarding lst after fees
    let lst_to_redeem_amount_sol_value = SolToLstRetVal(cpi_sol_to_lst(
        cpi,
        abr,
        lst_calc_prog,
        lp_tokens_sol_value_after_fees.0,
        SvcIxAccountHandles::new(
            NewSvcIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .build(),
            lst_calc,
        ),
    )?);

    // If lp token sol value + fees is greater than og value Will dilute existing LPs if unchecked.
    if lp_tokens_sol_value_after_fees.0 > lp_tokens_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    let pool_lst_reserve_balance =
        RawTokenAccount::of_acc_data(abr.get(*ix_prefix.pool_reserves()).data())
            .and_then(TokenAccount::try_from_raw)
            .map(|a| a.amount())
            .ok_or(INVALID_ACCOUNT_DATA)?;

    let pool_protocol_fee_bps =
        unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?
            .lp_protocol_fee_bps;

    let remove_liquidity_quote = quote_remove_liq(RemoveLiqQuoteArgs {
        amt: ix_args.amount,
        lp_token_supply,
        pool_total_sol_value: pool_sol_value,
        out_reserves: pool_lst_reserve_balance,
        lp_protocol_fee_bps: pool_protocol_fee_bps,
        out_mint: *abr.get(*ix_prefix.lst_mint()).key(),
        lp_mint: *abr.get(*ix_prefix.lp_token_mint()).key(),
        out_calc: lst_to_redeem_amount_sol_value,
        pricing: lp_tokens_sol_value_after_fees,
    })
    .map_err(|e| ProgramError::from(RemoveLiqQuoteProgErr(e)))?;

    let to_user_lst_amount = remove_liquidity_quote.0.out;

    if to_user_lst_amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    if to_user_lst_amount < ix_args.min_out {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    // Burning new LSTs based on deposit amount

    let lp_token_prog = *abr.get(*ix_prefix.lp_token_program()).key();

    let burn_accounts = NewBurnIxAccsBuilder::start()
        .with_auth(*ix_prefix.signer())
        .with_mint(*ix_prefix.lp_token_mint())
        .with_from(*ix_prefix.lp_acc())
        .build();

    let ix_data = BurnIxData::new(ix_args.amount);

    cpi.invoke_fwd(abr, &lp_token_prog, ix_data.as_buf(), burn_accounts.0)?;

    let lst_mint_acc_data = abr.get(*ix_prefix.lst_mint()).data();
    let lst_mint_decimals = RawMint::of_acc_data(lst_mint_acc_data)
        .and_then(Mint::try_from_raw)
        .map(|a| a.decimals())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let token_prog = *abr.get(*ix_prefix.lst_mint()).owner();

    for (dst, amt) in [
        (ix_prefix.lst_acc(), to_user_lst_amount),
        (
            ix_prefix.protocol_fee_accumulator(),
            remove_liquidity_quote.0.protocol_fee,
        ),
    ] {
        let transfer_checked_accounts = NewTransferCheckedIxAccsBuilder::start()
            .with_auth(*ix_prefix.pool_state())
            .with_src(*ix_prefix.pool_reserves())
            .with_dst(*dst)
            .with_mint(*ix_prefix.lst_mint())
            .build();

        let ix_data = TransferCheckedIxData::new(amt, lst_mint_decimals);

        let transfer_perms = transfer_checked_ix_account_handle_perms(transfer_checked_accounts);

        cpi.invoke_signed(
            abr,
            &token_prog,
            ix_data.as_buf(),
            transfer_perms,
            &[POOL_SEED_SIGNER],
        )?;
    }

    lst_sync_sol_val_unchecked(abr, cpi, sync_sol_val_calcs, ix_args.lst_index as usize)?;

    Ok(())
}

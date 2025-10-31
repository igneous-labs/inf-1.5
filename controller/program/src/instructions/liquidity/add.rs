use crate::svc::lst_sync_sol_val_unchecked;
#[allow(deprecated)]
use inf1_core::{
    instructions::liquidity::add::AddLiquidityIxAccs,
    quote::liquidity::add::{quote_add_liq, AddLiqQuoteArgs, AddLiqQuoteErr},
};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    cpi::{AddLiquidityPreAccountHandles, LstToSolRetVal, PricingRetVal},
    err::Inf1CtlErr,
    instructions::{
        liquidity::{
            add::{AddLiquidityIxArgs, NewAddLiquidityIxPreAccsBuilder},
            IxPreAccs,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_BUMP, POOL_STATE_ID},
    pda::POOL_STATE_SEED,
    pda_onchain::{create_raw_pool_reserves_addr, create_raw_protocol_fee_accumulator_addr},
    program_err::Inf1CtlCustomProgErr,
};
use inf1_pp_jiminy::{
    cpi::price::lp::{cpi_price_exact_in, IxAccountHandles as PriceInIxAccountHandles},
    instructions::price::exact_in::PriceExactInIxArgs,
};

use inf1_std::instructions::sync_sol_value::SyncSolValueIxAccs;

use inf1_svc_ag_core::inf1_svc_spl_core::sanctum_spl_stake_pool_core::TOKEN_PROGRAM;
use inf1_svc_jiminy::{
    cpi::{cpi_lst_to_sol, IxAccountHandles as SvcIxAccountHandles},
    instructions::NewIxPreAccsBuilder as NewSvcIxPreAccsBuilder,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    pda::{PdaSeed, PdaSigner},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
    Cpi,
};
use jiminy_log::sol_log;
use sanctum_spl_token_jiminy::{
    instructions::mint_to::mint_to_ix_account_handle_perms,
    sanctum_spl_token_core::{
        instructions::{
            mint_to::{MintToIxData, NewMintToIxAccsBuilder},
            transfer::{NewTransferCheckedIxAccsBuilder, TransferCheckedIxData},
        },
        state::{
            account::{RawTokenAccount, TokenAccount},
            mint::{Mint, RawMint},
        },
    },
};

use crate::pricing_program::NewPPIxPreAccsBuilder;

use crate::verify::{
    verify_not_input_disabled, verify_not_rebalancing_and_not_disabled, verify_pks,
};

#[allow(deprecated)]
pub type AddLiquidityIxAccounts<'a, 'acc> = AddLiquidityIxAccs<
    AccountHandle<'acc>,
    AddLiquidityPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
>;

/// Returns (prefix, sol_val_calc_program, remaining accounts, pricing_program, remaining accounts)
#[inline]
fn add_liquidity_accs_checked<'a, 'acc>(
    abr: &Abr,
    accounts: &'a [AccountHandle<'acc>],
    ix_args: AddLiquidityIxArgs,
) -> Result<AddLiquidityIxAccounts<'a, 'acc>, ProgramError> {
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

    let expected_pks = NewAddLiquidityIxPreAccsBuilder::start()
        // These can be arbitrary accs bc they belong to the user adding liquidity to INF
        .with_signer(abr.get(*ix_prefix.signer()).key())
        .with_lst_mint(&lst_state.mint)
        .with_lst_acc(abr.get(*ix_prefix.lst_acc()).key())
        .with_lp_acc(abr.get(*ix_prefix.lp_acc()).key())
        .with_lp_token_mint(&pool.lp_token_mint)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_lst_token_program(&TOKEN_PROGRAM)
        .with_lp_token_program(&TOKEN_PROGRAM)
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_reserves(&expected_reserves)
        .build();

    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;
    sol_log("after verify prefix");

    let (lst_cal_all, pricing_all) = suf
        // Adding +1 here bc we need to take into account the program as well
        .split_at_checked((ix_args.lst_value_calc_accs + 1).into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let (lst_calc_prog, lst_calc_acc) = lst_cal_all.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let (pricing_prog, pricing_accs) = pricing_all.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    verify_pks(
        abr,
        &[*lst_calc_prog, *pricing_prog],
        &[&lst_state.sol_value_calculator, &pool.pricing_program],
    )?;

    verify_not_rebalancing_and_not_disabled(pool)?;
    verify_not_input_disabled(lst_state)?;

    Ok(AddLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog: *lst_calc_prog,
        lst_calc: lst_calc_acc,
        pricing_prog: *pricing_prog,
        pricing: pricing_accs,
    })
}

#[inline]
#[allow(deprecated)]
pub fn process_add_liquidity(
    abr: &mut Abr,
    accounts: &[AccountHandle],
    ix_args: AddLiquidityIxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    sol_log("Processing");
    let AddLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog,
        lst_calc,
        pricing_prog,
        pricing,
    } = add_liquidity_accs_checked(abr, accounts, ix_args)?;

    let lp_lst_supply = RawTokenAccount::of_acc_data(abr.get(*ix_prefix.pool_reserves()).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;
    sol_log("lst_sync_sol_val_unchecked");

    lst_sync_sol_val_unchecked(
        abr,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.pool_reserves())
                .build(),
            calc_prog: lst_calc_prog,
            calc: lst_calc,
        },
        ix_args.lst_index as usize,
    )?;
    sol_log("after lst_sync_sol_val_unchecked");

    // Extract the data you need from pool before CPI calls
    let start_total_sol_value = unsafe {
        PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data())
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?
            .total_sol_value
    };

    // Step 4: Calculate sol_value_to_add = LstToSol(amount).min
    sol_log("cpi_lst_to_soln2");

    let lst_amount_sol_value = LstToSolRetVal(cpi_lst_to_sol(
        cpi,
        abr,
        lst_calc_prog,
        ix_args.amount,
        SvcIxAccountHandles::new(
            NewSvcIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .build(),
            lst_calc,
        ),
    )?);
    sol_log("cpi_price_exact_in");
    // Step 5: Calculate sol_value_to_add_after_fees = PriceLpTokensToMint(lp_tokens_sol_value)
    let lst_amount_sol_value_after_fees = PricingRetVal(cpi_price_exact_in(
        cpi,
        abr,
        pricing_prog,
        PriceExactInIxArgs {
            sol_value: *lst_amount_sol_value.0.end(),
            amt: ix_args.amount,
        },
        PriceInIxAccountHandles::new(
            NewPPIxPreAccsBuilder::start()
                .with_input_mint(*ix_prefix.lst_mint())
                .with_output_mint(*ix_prefix.lp_token_mint())
                .build(),
            pricing,
        ),
    )?);
    sol_log(&format!("{:?}", ix_args));
    sol_log(&format!("lst_balance {:?}", lp_lst_supply));
    sol_log(&format!(
        "lst_amount_sol_value_after_fees {:?}",
        lst_amount_sol_value_after_fees.0
    ));
    sol_log(&format!(
        "lst_amount_sol_value {:?}",
        lst_amount_sol_value.0
    ));

    let pool = unsafe { PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    // Will dilute existing LPs if unchecked
    if lst_amount_sol_value_after_fees.0 > *lst_amount_sol_value.0.end() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }
    sol_log(&format!("lp_lst_supply {:?}", lp_lst_supply));
    sol_log(&format!(
        "lst_amount_sol_value_after_fees {:?}",
        lst_amount_sol_value_after_fees.0
    ));
    sol_log(&format!(
        "lst_amount_sol_value {:?}",
        lst_amount_sol_value.0
    ));
    sol_log(&format!("pool.total_sol_value {:?}", pool.total_sol_value));
    sol_log(&format!(
        "pool.pool.lp_protocol_fee_bps {:?}",
        pool.lp_protocol_fee_bps
    ));
    sol_log("Add liquidity");
    let add_liquidity_quote = match quote_add_liq(AddLiqQuoteArgs {
        amt: ix_args.amount,
        lp_token_supply: lp_lst_supply,
        lp_mint: pool.lp_token_mint,
        lp_protocol_fee_bps: pool.lp_protocol_fee_bps,
        pool_total_sol_value: pool.total_sol_value,
        inp_calc: lst_amount_sol_value,
        pricing: lst_amount_sol_value_after_fees,
        inp_mint: *abr.get(*ix_prefix.lst_mint()).key(),
    }) {
        Ok(quote) => quote,
        Err(error) => match error {
            AddLiqQuoteErr::Overflow => {
                return Err(Inf1CtlCustomProgErr(Inf1CtlErr::MathError).into())
            }
            AddLiqQuoteErr::ZeroValue => {
                return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into())
            }
            AddLiqQuoteErr::InpCalc(x) => return Err(x),
            AddLiqQuoteErr::Pricing(x) => return Err(x),
        },
    };
    // Step 6: lp_fees_sol_value = lp_tokens_sol_value - sol_value_to_add_after_fees
    if add_liquidity_quote.0.lp_fee == 0 || add_liquidity_quote.0.out == 0 {
        sol_log("hereeee");
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    sol_log(&format!("{:?}", add_liquidity_quote.0));

    if add_liquidity_quote.0.out < ix_args.min_out {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let transfer_checked_accounts = NewTransferCheckedIxAccsBuilder::start()
        .with_auth(*ix_prefix.signer())
        .with_src(*ix_prefix.lst_acc())
        .with_dst(*ix_prefix.pool_reserves())
        .with_mint(*ix_prefix.lst_mint())
        .build();

    let lst_mint_acc_data = abr.get(*ix_prefix.lst_mint()).data();
    let lst_mint_decimals = RawMint::of_acc_data(lst_mint_acc_data)
        .and_then(Mint::try_from_raw)
        .map(|a| a.decimals())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let token_prog = *abr.get(*ix_prefix.lst_mint()).owner();

    let ix_data = TransferCheckedIxData::new(add_liquidity_quote.0.lp_fee, lst_mint_decimals);

    // Transferring deposit fees to pool reserves
    cpi.invoke_fwd(
        abr,
        &token_prog,
        ix_data.as_buf(),
        transfer_checked_accounts.0,
    )?;

    let transfer_checked_accounts = NewTransferCheckedIxAccsBuilder::start()
        .with_auth(*ix_prefix.signer())
        .with_src(*ix_prefix.lst_acc())
        .with_dst(*ix_prefix.protocol_fee_accumulator())
        .with_mint(*ix_prefix.lst_mint())
        .build();

    let ix_data = TransferCheckedIxData::new(add_liquidity_quote.0.protocol_fee, lst_mint_decimals);

    // Transferring deposit fees to protrocol
    cpi.invoke_fwd(
        abr,
        &token_prog,
        ix_data.as_buf(),
        transfer_checked_accounts.0,
    )?;

    // Minting new LSTs based on deposit amount

    let lp_token_prog = *abr.get(*ix_prefix.lp_token_program()).key();

    let mint_checked_accounts = NewMintToIxAccsBuilder::start()
        .with_auth(*ix_prefix.pool_state())
        .with_mint(*ix_prefix.lp_token_mint())
        .with_to(*ix_prefix.lp_acc())
        .build();

    let ix_data = MintToIxData::new(add_liquidity_quote.0.out);

    let mint_perms = mint_to_ix_account_handle_perms(mint_checked_accounts);

    cpi.invoke_signed(
        abr,
        &lp_token_prog,
        ix_data.as_buf(),
        mint_perms,
        &[PdaSigner::new(&[
            PdaSeed::new(POOL_STATE_SEED.as_slice()),
            PdaSeed::new(&[POOL_STATE_BUMP]),
        ])],
    )?;

    lst_sync_sol_val_unchecked(
        abr,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.pool_reserves())
                .build(),
            calc_prog: lst_calc_prog,
            calc: lst_calc,
        },
        ix_args.lst_index as usize,
    )?;

    let end_total_sol_value = unsafe {
        PoolState::of_acc_data(abr.get(*ix_prefix.pool_state()).data())
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?
            .total_sol_value
    };

    if end_total_sol_value < start_total_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    Ok(())
}

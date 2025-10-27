use std::{fmt::format, ops::Range};

use crate::{svc::lst_sync_sol_val_unchecked, Accounts};
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
    cpi::price::lp::cpi_price_exact_in,
    instructions::{deprecated::lp::mint, price::exact_in::PriceExactInIxArgs},
    traits::{deprecated::PriceLpTokensToMint, main::PriceExactIn},
};

use inf1_std::instructions::sync_sol_value::SyncSolValueIxAccs;

use inf1_svc_ag_core::inf1_svc_spl_core::sanctum_spl_stake_pool_core::TOKEN_PROGRAM;
use inf1_svc_jiminy::cpi::cpi_lst_to_sol;
use jiminy_cpi::{
    account::AccountHandle,
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
use solana_pubkey::Pubkey;

use crate::pricing_program::NewPPIxPreAccsBuilder;
use crate::svc::NewSvcIxPreAccsBuilder;
use crate::verify::{
    verify_not_input_disabled, verify_not_rebalancing_and_not_disabled, verify_pks,
};

pub type AddLiquidityIxAccounts<'acc> = AddLiquidityIxAccs<
    AccountHandle<'acc>,
    AddLiquidityPreAccountHandles<'acc>,
    Range<usize>,
    Range<usize>,
>;

/// Returns (prefix, sol_val_calc_program, remaining accounts, pricing_program, remaining accounts)

#[inline]
fn add_liquidity_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
    ix_args: AddLiquidityIxArgs,
) -> Result<AddLiquidityIxAccounts<'acc>, ProgramError> {
    if ix_args.amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    let (ix_prefix, suf) = accounts
        .as_slice()
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let ix_prefix = IxPreAccs(*ix_prefix);
    let list = LstStatePackedList::of_acc_data(accounts.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let lst_state = list
        .0
        .get(ix_args.lst_index as usize)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let lst_mint_acc = accounts.get(*ix_prefix.lst_mint());
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
        .with_signer(accounts.get(*ix_prefix.signer()).key())
        .with_lst_mint(&lst_state.mint)
        .with_lst_acc(accounts.get(*ix_prefix.lst_acc()).key())
        .with_lp_acc(accounts.get(*ix_prefix.lp_acc()).key())
        .with_lp_token_mint(&pool.lp_token_mint)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_lst_token_program(&TOKEN_PROGRAM)
        .with_lp_token_program(&TOKEN_PROGRAM)
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_reserves(&expected_reserves)
        .build();

    verify_pks(accounts, &ix_prefix.0, &expected_pks.0)?;
    let calc_prog = suf.first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(accounts, &[*calc_prog], &[&lst_state.sol_value_calculator])?;

    // Taking out prog addres and calculating the number of accounts for lst_calc_program
    let calc_end = 1 + ix_args.lst_value_calc_accs as usize - 1;

    // +1 to skip program
    let pricing_start = calc_end + 1;

    // Get pricing program, first account after lst_calc_acc
    let pricing_prog = suf.get(pricing_start).ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    verify_pks(accounts, &[*pricing_prog], &[&pool.pricing_program])?;

    verify_not_rebalancing_and_not_disabled(&pool)?;
    verify_not_input_disabled(&lst_state)?;

    let suf_acc_start_idx = ix_prefix.0.len() + 1;

    Ok(AddLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog: *calc_prog,
        // + 1 to skip the program
        lst_calc: suf_acc_start_idx..(suf_acc_start_idx + calc_end),
        pricing_prog: *pricing_prog,
        pricing: (suf_acc_start_idx + pricing_start)..accounts.as_slice().len(),
    })
}

#[inline]
#[allow(deprecated)]
pub fn process_add_liquidity(
    accounts: &mut Accounts<'_>,
    ix_args: AddLiquidityIxArgs,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let AddLiquidityIxAccounts {
        ix_prefix,
        lst_calc_prog,
        lst_calc,
        pricing_prog,
        pricing,
    } = add_liquidity_accs_checked(accounts, ix_args)?;
    sol_log(&format!("Length of lst calc{:?}", &lst_calc));
    sol_log(&format!("Length of pricing{:?}", pricing));

    let a = accounts.as_slice().get(pricing.clone()).unwrap()[0];
    sol_log(&format!("Price account is {:?}", *accounts.get(a).key()));
    sol_log("Checked");

    let lst_balance = RawTokenAccount::of_acc_data(accounts.get(*ix_prefix.pool_reserves()).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;
    sol_log("syncing");

    lst_sync_sol_val_unchecked(
        accounts,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.pool_reserves())
                .build(),
            calc_prog: lst_calc_prog,
            calc: lst_calc.clone(),
        },
        ix_args.lst_index as usize,
    )?;
    sol_log("Sol synced");
    // Extract the data you need from pool before CPI calls
    let start_total_sol_value = unsafe {
        PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data())
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?
            .total_sol_value
    };

    // Step 4: Calculate sol_value_to_add = LstToSol(amount).min

    let lst_amount_sol_value = LstToSolRetVal(cpi_lst_to_sol(
        cpi,
        accounts,
        lst_calc_prog,
        ix_args.amount,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.lst_mint())
            .build(),
        &lst_calc,
    )?);

    sol_log("value after fees pre");

    // Step 5: Calculate sol_value_to_add_after_fees = PriceLpTokensToMint(lp_tokens_sol_value)
    let lst_amount_sol_value_after_fees = PricingRetVal(cpi_price_exact_in(
        cpi,
        accounts,
        pricing_prog,
        PriceExactInIxArgs {
            sol_value: *lst_amount_sol_value.0.end(),
            amt: ix_args.amount,
        },
        NewPPIxPreAccsBuilder::start()
            .with_input_mint(*ix_prefix.lst_mint())
            .with_output_mint(*ix_prefix.lp_token_mint())
            .build(),
        pricing,
    )?);

    sol_log(&format!(
        "value after fees post {:?}",
        lst_amount_sol_value_after_fees.0
    ));

    sol_log(&format!(
        "value after fees post {:?}",
        lst_amount_sol_value.0
    ));

    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    // Will dilute existing LPs if unchecked
    if lst_amount_sol_value_after_fees.0 > *lst_amount_sol_value.0.end() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    sol_log("Quote add liq");
    let add_liquidity_quote = match quote_add_liq(AddLiqQuoteArgs {
        amt: ix_args.amount,
        lp_token_supply: lst_balance,
        lp_mint: pool.lp_token_mint,
        lp_protocol_fee_bps: pool.lp_protocol_fee_bps,
        pool_total_sol_value: pool.total_sol_value,
        inp_calc: lst_amount_sol_value,
        pricing: lst_amount_sol_value_after_fees,
        inp_mint: *accounts.get(*ix_prefix.lst_mint()).key(),
    }) {
        Ok(quote) => quote,
        //Redo this for compatibility
        Err(error) => match error {
            AddLiqQuoteErr::Overflow => {
                return Err(Inf1CtlCustomProgErr(Inf1CtlErr::MathError).into())
            }
            AddLiqQuoteErr::ZeroValue => {
                return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into())
            }
            AddLiqQuoteErr::InpCalc(x) => return Err(x.into()),
            AddLiqQuoteErr::Pricing(x) => return Err(x.into()),
        },
    };

    sol_log(&format!("Quote {:#?}", add_liquidity_quote));

    // Step 6: lp_fees_sol_value = lp_tokens_sol_value - sol_value_to_add_after_fees
    if add_liquidity_quote.0.lp_fee == 0 || add_liquidity_quote.0.out == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    if add_liquidity_quote.0.out < ix_args.min_out {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded).into());
    }

    let transfer_checked_accounts = NewTransferCheckedIxAccsBuilder::start()
        .with_auth(*ix_prefix.signer())
        .with_src(*ix_prefix.lst_acc())
        .with_dst(*ix_prefix.pool_reserves())
        .with_mint(*ix_prefix.lst_mint())
        .build();

    let lst_mint_acc_data = accounts.get(*ix_prefix.lst_mint()).data();
    let lst_mint_decimals = RawMint::of_acc_data(lst_mint_acc_data)
        .and_then(Mint::try_from_raw)
        .map(|a| a.decimals())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let token_prog = *accounts.get(*ix_prefix.lst_mint()).owner();

    let ix_data = TransferCheckedIxData::new(add_liquidity_quote.0.lp_fee, lst_mint_decimals);

    // Transferring deposit fees to pool reserves
    sol_log("Transferring deposit fees to pool reserves");
    cpi.invoke_fwd(
        accounts,
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
    sol_log("Transferring deposit fees to protocol");

    cpi.invoke_fwd(
        accounts,
        &token_prog,
        ix_data.as_buf(),
        transfer_checked_accounts.0,
    )?;

    // Minting new LSTs based on deposit amount
    sol_log("Minting");

    let lp_token_prog = *accounts.get(*ix_prefix.lp_token_program()).key();
    let mint = *accounts.get(*ix_prefix.lp_token_mint()).key();

    sol_log(&format!("mint is {:?}", Pubkey::new_from_array(mint)));
    sol_log(&format!(
        "lp_acc is {:?}",
        Pubkey::new_from_array(*accounts.get(*ix_prefix.lp_acc()).key())
    ));

    let mint_checked_accounts = NewMintToIxAccsBuilder::start()
        .with_auth(*ix_prefix.pool_state())
        .with_mint(*ix_prefix.lp_token_mint())
        .with_to(*ix_prefix.lp_acc())
        .build();

    let ix_data = MintToIxData::new(add_liquidity_quote.0.out);

    let mint_perms = mint_to_ix_account_handle_perms(mint_checked_accounts);

    cpi.invoke_signed(
        accounts,
        &lp_token_prog,
        ix_data.as_buf(),
        mint_perms,
        &[PdaSigner::new(&[
            PdaSeed::new(POOL_STATE_SEED.as_slice()),
            PdaSeed::new(&[POOL_STATE_BUMP]),
        ])],
    )?;

    lst_sync_sol_val_unchecked(
        accounts,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.pool_reserves())
                .build(),
            calc_prog: lst_calc_prog,
            calc: lst_calc.clone(),
        },
        ix_args.lst_index as usize,
    )?;

    let end_total_sol_value = unsafe {
        PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data())
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?
            .total_sol_value
    };

    if end_total_sol_value < start_total_sol_value {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
    }

    Ok(())
}

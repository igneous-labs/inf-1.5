use crate::{
    common::SVM,
    tests::rebalance::test_utils::{
        add_common_accounts, assert_balanced, fixture_lst_state_data, jupsol_wsol_builder,
        rebalance_ixs, StartRebalanceKeysBuilder,
    },
};

use inf1_ctl_jiminy::{
    accounts::{
        pool_state::{PoolState, PoolStatePacked},
        rebalance_record::RebalanceRecord,
    },
    keys::{INSTRUCTIONS_SYSVAR_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
};

use inf1_svc_ag_core::inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM;

use inf1_test_utils::{
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, mock_instructions_sysvar,
    mock_mint, mock_token_acc, raw_mint, raw_token_acc, silence_mollusk_logs, upsert_account,
    PkAccountTup,
};

use mollusk_svm::{
    program::keyed_account_for_system_program,
    result::{Check, InstructionResult, ProgramResult},
};

use sanctum_spl_token_jiminy::sanctum_spl_token_core::{
    instructions::transfer::{
        NewTransferCheckedIxAccsBuilder, TransferCheckedIxData, TRANSFER_CHECKED_IX_IS_SIGNER,
        TRANSFER_CHECKED_IX_IS_WRITABLE,
    },
    state::mint::{Mint, RawMint},
};

use sanctum_system_jiminy::sanctum_system_core::ID as SYSTEM_PROGRAM_ID;

use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use std::collections::HashMap;

use proptest::prelude::*;

fn create_transfer_ix(
    owner: Pubkey,
    owner_token_account: Pubkey,
    inp_mint: [u8; 32],
    inp_mint_decimals: u8,
    inp_pool_reserves: Pubkey,
    amount: u64,
) -> Instruction {
    let transfer_ix_data = TransferCheckedIxData::new(amount, inp_mint_decimals);
    let transfer_accs = NewTransferCheckedIxAccsBuilder::start()
        .with_src(owner_token_account.to_bytes())
        .with_mint(inp_mint)
        .with_dst(inp_pool_reserves.to_bytes())
        .with_auth(owner.to_bytes())
        .build();

    Instruction {
        program_id: Pubkey::new_from_array(TOKENKEG_PROGRAM),
        accounts: keys_signer_writable_to_metas(
            transfer_accs.0.iter(),
            TRANSFER_CHECKED_IX_IS_SIGNER.0.iter(),
            TRANSFER_CHECKED_IX_IS_WRITABLE.0.iter(),
        ),
        data: transfer_ix_data.as_buf().into(),
    }
}

/// Creates the full account set required for StartRebalance → Token Transfer → EndRebalance transaction
#[allow(clippy::too_many_arguments)]
fn setup_rebalance_transaction_accounts(
    builder: &StartRebalanceKeysBuilder,
    instructions: &[Instruction],
    pool: &PoolState,
    lst_state_list: &[u8],
    pool_reserves_map: &HashMap<[u8; 32], [u8; 32]>,
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
    out_balance: u64,
    inp_balance: u64,
    owner: Pubkey,
    owner_token_account: Pubkey,
    owner_balance: u64,
) -> Vec<PkAccountTup> {
    let mut accounts: Vec<PkAccountTup> =
        fixtures_accounts_opt_cloned(builder.keys_owned().seq().copied()).collect();

    add_common_accounts(
        &mut accounts,
        pool,
        lst_state_list,
        Some(pool_reserves_map),
        rebalance_auth,
        out_mint,
        inp_mint,
        withdraw_to,
        out_balance,
        inp_balance,
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
            keyed_account_for_system_program().1,
        ),
    );

    upsert_account(
        &mut accounts,
        (
            owner,
            Account {
                lamports: 100_000_000_000,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Default::default()
            },
        ),
    );

    upsert_account(
        &mut accounts,
        (
            owner_token_account,
            mock_token_acc(raw_token_acc(inp_mint, owner.to_bytes(), owner_balance)),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(INSTRUCTIONS_SYSVAR_ID),
            mock_instructions_sysvar(instructions, 0),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(REBALANCE_RECORD_ID),
            Account::default(),
        ),
    );

    accounts
}

/// Creates an instruction chain with StartRebalance → Token Transfer → EndRebalance instructions.
#[allow(clippy::too_many_arguments)]
fn build_rebalance_instruction_chain(
    builder: &StartRebalanceKeysBuilder,
    out_idx: u32,
    inp_idx: u32,
    out_lst_amount: u64,
    owner_transfer_amount: u64,
    owner: Pubkey,
    owner_token_account: Pubkey,
    inp_mint: [u8; 32],
    inp_mint_account: &Account,
    inp_pool_reserves: Pubkey,
) -> Vec<Instruction> {
    let mut instructions = rebalance_ixs(builder, out_idx, inp_idx, out_lst_amount, 0, u64::MAX);

    let inp_mint_decimals = RawMint::of_acc_data(&inp_mint_account.data)
        .and_then(Mint::try_from_raw)
        .expect("valid mint")
        .decimals();

    let transfer_ix = create_transfer_ix(
        owner,
        owner_token_account,
        inp_mint,
        inp_mint_decimals,
        inp_pool_reserves,
        owner_transfer_amount,
    );

    // Put transfer ix between start and end
    instructions.insert(1, transfer_ix);

    instructions
}

fn execute_rebalance_transaction(amount: u64) -> (Vec<PkAccountTup>, InstructionResult, u64) {
    silence_mollusk_logs();

    let (pool, mut lsl, mut out_lsd, mut inp_lsd) = fixture_lst_state_data();

    let withdraw_to = Pubkey::new_unique().to_bytes();
    let builder = jupsol_wsol_builder(
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    out_lsd.lst_state.sol_value_calculator = builder.out_calc_prog;
    inp_lsd.lst_state.sol_value_calculator = builder.inp_calc_prog;

    let out_idx = lsl.upsert(out_lsd) as u32;
    let inp_idx = lsl.upsert(inp_lsd) as u32;

    let owner = Pubkey::new_unique();
    let owner_token_account = Pubkey::new_unique();

    let inp_mint_account = mock_mint(raw_mint(None, None, 0, 9));
    let inp_mint_decimals = RawMint::of_acc_data(&inp_mint_account.data)
        .and_then(Mint::try_from_raw)
        .expect("valid mint")
        .decimals();

    let inp_pool_reserves = Pubkey::new_from_array(inp_lsd.pool_reserves);

    let preview_ixs = rebalance_ixs(&builder, out_idx, inp_idx, amount, 0, u64::MAX);

    let transfer_ix = create_transfer_ix(
        owner,
        owner_token_account,
        inp_lsd.lst_state.mint,
        inp_mint_decimals,
        inp_pool_reserves,
        0,
    );
    let preview_chain = vec![preview_ixs[0].clone(), transfer_ix, preview_ixs[1].clone()];

    let preview_accounts = setup_rebalance_transaction_accounts(
        &builder,
        &preview_chain,
        &pool,
        &lsl.lst_state_list,
        &lsl.all_pool_reserves,
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
        amount * 2,
        amount * 2,
        owner,
        owner_token_account,
        0,
    );

    let start_result =
        SVM.with(|svm| svm.process_instruction(&preview_chain[0], &preview_accounts));

    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let rr_pk = Pubkey::new_from_array(REBALANCE_RECORD_ID);

    let pool_after_start = start_result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == pool_pk)
        .map(|(_, acc)| acc)
        .expect("pool after start");

    let rr_after_start = start_result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == rr_pk)
        .map(|(_, acc)| acc)
        .expect("rebalance record after start");

    let pool_state_after = PoolStatePacked::of_acc_data(&pool_after_start.data)
        .expect("pool state")
        .into_pool_state();

    let rebalance_record =
        unsafe { RebalanceRecord::of_acc_data(&rr_after_start.data) }.expect("rebalance record");

    // Calculate how many lamports the pool lost
    let deficit = rebalance_record
        .old_total_sol_value
        .saturating_sub(pool_state_after.total_sol_value);

    let owner_transfer_amount = deficit + (deficit / 100);

    let instructions = build_rebalance_instruction_chain(
        &builder,
        out_idx,
        inp_idx,
        amount,
        owner_transfer_amount,
        owner,
        owner_token_account,
        inp_lsd.lst_state.mint,
        &inp_mint_account,
        inp_pool_reserves,
    );

    let accounts = setup_rebalance_transaction_accounts(
        &builder,
        &instructions,
        &pool,
        &lsl.lst_state_list,
        &lsl.all_pool_reserves,
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
        amount * 2,
        amount * 2,
        owner,
        owner_token_account,
        owner_transfer_amount,
    );

    let accounts_before = accounts.clone();

    let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accounts));

    (
        accounts_before,
        result,
        rebalance_record.old_total_sol_value,
    )
}

/// Validate that the transaction succeeded,
/// the pool state is not rebalancing before or after,
/// the lamports are balanced,
/// the pool did not lose SOL value,
/// and the RebalanceRecord is properly closed.
fn assert_rebalance_transaction_success(
    accounts_before: &[PkAccountTup],
    result: &InstructionResult,
    old_total_sol_value: u64,
) {
    assert_eq!(result.program_result, ProgramResult::Success);

    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_bef = accounts_before
        .iter()
        .find(|(pk, _)| *pk == pool_pk)
        .map(|(_, acc)| acc)
        .expect("pool before");
    let pool_aft = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == pool_pk)
        .map(|(_, acc)| acc)
        .expect("pool after");

    let pool_state_bef = PoolStatePacked::of_acc_data(&pool_bef.data)
        .expect("pool before")
        .into_pool_state();
    let pool_state_aft = PoolStatePacked::of_acc_data(&pool_aft.data)
        .expect("pool after")
        .into_pool_state();

    assert_eq!(pool_state_bef.is_rebalancing, 0);
    assert_eq!(pool_state_aft.is_rebalancing, 0);
    assert!(pool_state_aft.total_sol_value >= old_total_sol_value,);

    // Assert RebalanceRecord is closed
    let rr_pk = Pubkey::new_from_array(REBALANCE_RECORD_ID);
    let rr_after = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == rr_pk);
    assert!(rr_after.is_none() || rr_after.unwrap().1.lamports == 0);

    assert_balanced(accounts_before, &result.resulting_accounts);
}

#[test]
fn rebalance_transaction_success() {
    let amount = 100_000;
    let (accounts_before, result, old_total_sol_value) = execute_rebalance_transaction(amount);

    assert_rebalance_transaction_success(&accounts_before, &result, old_total_sol_value);

    let mut result_for_check = result.clone();

    if let Some(token_prog_before) = accounts_before
        .iter()
        .find(|(pk, _)| pk.to_bytes() == TOKENKEG_PROGRAM)
    {
        if let Some(token_prog_result) = result_for_check
            .resulting_accounts
            .iter_mut()
            .find(|(pk, _)| pk.to_bytes() == TOKENKEG_PROGRAM)
        {
            token_prog_result.1 = token_prog_before.1.clone();
        }
    }

    if let Some(instructions_sysvar_before) = accounts_before
        .iter()
        .find(|(pk, _)| pk.to_bytes() == INSTRUCTIONS_SYSVAR_ID)
    {
        if let Some(instructions_sysvar_result) = result_for_check
            .resulting_accounts
            .iter_mut()
            .find(|(pk, _)| pk.to_bytes() == INSTRUCTIONS_SYSVAR_ID)
        {
            instructions_sysvar_result.1 = instructions_sysvar_before.1.clone();
        }
    }

    // Assert all accounts are rent-exempt after transaction
    SVM.with(|svm| {
        assert!(result_for_check.run_checks(&[Check::all_rent_exempt()], &svm.config, svm));
    });
}

proptest! {
    #[test]
    fn rebalance_transaction_various_amounts_any(
        amount in 1_000u64..=1_000_000
    ) {
        let (accounts_before, result, old_total_sol_value) = execute_rebalance_transaction(amount);

        assert_rebalance_transaction_success(&accounts_before, &result, old_total_sol_value);
    }
}

use crate::{
    common::SVM,
    tests::rebalance::test_utils::{
        add_common_accounts, fixture_lst_state_data, jupsol_wsol_builder, rebalance_ixs,
        StartRebalanceKeysBuilder,
    },
};

use inf1_test_utils::{LstStateData, LstStateListData};

use inf1_core::quote::rebalance::{quote_rebalance_exact_out, RebalanceQuoteArgs};

use inf1_ctl_jiminy::{
    accounts::{
        pool_state::{PoolState, PoolStatePacked},
        rebalance_record::RebalanceRecord,
    },
    err::Inf1CtlErr::{
        NoSucceedingEndRebalance, PoolRebalancing, PoolWouldLoseSolValue, SlippageToleranceExceeded,
    },
    instructions::rebalance::end::END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT,
    keys::{INSTRUCTIONS_SYSVAR_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
    program_err::Inf1CtlCustomProgErr,
};

use inf1_svc_ag_core::inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM;

use inf1_svc_ag_core::{
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    inf1_svc_wsol_core::calc::WsolCalc,
};

use inf1_test_utils::{
    acc_bef_aft, assert_balanced, assert_jiminy_prog_err, fixtures_accounts_opt_cloned,
    get_token_account_amount, keys_signer_writable_to_metas, mock_instructions_sysvar,
    mock_sys_acc, mock_token_acc, raw_token_acc, silence_mollusk_logs, upsert_account,
    KeyedUiAccount, PkAccountTup,
};

use jiminy_cpi::program_error::INVALID_ARGUMENT;

use mollusk_svm::{
    program::keyed_account_for_system_program,
    result::{Check, InstructionResult, ProgramResult},
};

use sanctum_spl_token_jiminy::sanctum_spl_token_core::instructions::transfer::{
    NewTransferIxAccsBuilder, TransferIxData, TRANSFER_IX_IS_SIGNER, TRANSFER_IX_IS_WRITABLE,
};

use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use proptest::prelude::*;

struct TestFixture {
    pool: PoolState,
    lsl: LstStateListData,
    out_lsd: LstStateData,
    inp_lsd: LstStateData,
    builder: StartRebalanceKeysBuilder,
    withdraw_to: [u8; 32],
    out_idx: u32,
    inp_idx: u32,
}

fn setup_test_fixture() -> TestFixture {
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

    TestFixture {
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        withdraw_to,
        out_idx,
        inp_idx,
    }
}

struct OwnerAccounts {
    owner: [u8; 32],
    owner_token_account: [u8; 32],
    owner_balance: u64,
}

fn setup_owner_accounts(balance: u64) -> OwnerAccounts {
    OwnerAccounts {
        owner: Pubkey::new_unique().to_bytes(),
        owner_token_account: Pubkey::new_unique().to_bytes(),
        owner_balance: balance,
    }
}

fn standard_reserves(amount: u64) -> (u64, u64) {
    (amount * 2, amount * 2)
}

fn setup_basic_rebalance_test(
    fixture: &TestFixture,
    amount: u64,
    min_starting_out_lst: u64,
    max_starting_inp_lst: u64,
) -> (Vec<Instruction>, Vec<PkAccountTup>) {
    let (out_reserves, inp_reserves) = standard_reserves(amount);
    let instructions = rebalance_ixs(
        &fixture.builder,
        fixture.out_idx,
        fixture.inp_idx,
        amount,
        min_starting_out_lst,
        max_starting_inp_lst,
    );
    let owner_accs = setup_owner_accounts(0);
    let accounts = setup_rebalance_transaction_accounts(
        fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );
    (instructions, accounts)
}

fn create_transfer_ix(
    owner: [u8; 32],
    owner_token_account: [u8; 32],
    inp_pool_reserves: [u8; 32],
    amount: u64,
) -> Instruction {
    let transfer_ix_data = TransferIxData::new(amount);
    let transfer_accs = NewTransferIxAccsBuilder::start()
        .with_src(owner_token_account)
        .with_dst(inp_pool_reserves)
        .with_auth(owner)
        .build();

    Instruction {
        program_id: Pubkey::new_from_array(TOKENKEG_PROGRAM),
        accounts: keys_signer_writable_to_metas(
            transfer_accs.0.iter(),
            TRANSFER_IX_IS_SIGNER.0.iter(),
            TRANSFER_IX_IS_WRITABLE.0.iter(),
        ),
        data: transfer_ix_data.as_buf().into(),
    }
}

/// Calculates the input token amount for a JupSOL -> WSOL rebalance
fn calculate_jupsol_wsol_inp_amount(
    out_lst_amount: u64,
    out_reserves: u64,
    inp_reserves: u64,
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
) -> u64 {
    let (_, jupsol_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pool.json").into_keyed_account();
    let jupsol_stakepool = StakePool::borsh_de(jupsol_pool_acc.data.as_slice()).unwrap();

    let inp_calc = WsolCalc;
    let out_calc = SplCalc::new(&jupsol_stakepool, 0);

    let quote = quote_rebalance_exact_out(RebalanceQuoteArgs {
        amt: out_lst_amount,
        inp_reserves,
        out_reserves,
        inp_mint,
        out_mint,
        inp_calc,
        out_calc,
    })
    .expect("quote should succeed");

    quote.inp
}

/// Creates the full account set required for StartRebalance → Token Transfer → EndRebalance transaction
fn setup_rebalance_transaction_accounts(
    fixture: &TestFixture,
    instructions: &[Instruction],
    out_balance: u64,
    inp_balance: u64,
    owner_accs: &OwnerAccounts,
) -> Vec<PkAccountTup> {
    let mut accounts: Vec<PkAccountTup> =
        fixtures_accounts_opt_cloned(fixture.builder.keys_owned().seq().copied()).collect();

    add_common_accounts(
        &mut accounts,
        &fixture.pool,
        &fixture.lsl.lst_state_list,
        Some(&fixture.lsl.all_pool_reserves),
        fixture.pool.rebalance_authority,
        fixture.out_lsd.lst_state.mint,
        fixture.inp_lsd.lst_state.mint,
        fixture.withdraw_to,
        out_balance,
        inp_balance,
    );

    upsert_account(&mut accounts, keyed_account_for_system_program());

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(owner_accs.owner),
            mock_sys_acc(100_000_000_000),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(owner_accs.owner_token_account),
            mock_token_acc(raw_token_acc(
                fixture.inp_lsd.lst_state.mint,
                owner_accs.owner,
                owner_accs.owner_balance,
            )),
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
fn build_rebalance_instruction_chain(
    fixture: &TestFixture,
    owner_accs: &OwnerAccounts,
    out_lst_amount: u64,
    owner_transfer_amount: u64,
) -> Vec<Instruction> {
    let mut instructions = rebalance_ixs(
        &fixture.builder,
        fixture.out_idx,
        fixture.inp_idx,
        out_lst_amount,
        0,
        u64::MAX,
    );

    let transfer_ix = create_transfer_ix(
        owner_accs.owner,
        owner_accs.owner_token_account,
        fixture.inp_lsd.pool_reserves,
        owner_transfer_amount,
    );

    // Put transfer ix between start and end
    instructions.insert(1, transfer_ix);

    instructions
}

fn execute_rebalance_transaction(
    amount: u64,
    out_reserves: Option<u64>,
    inp_reserves: Option<u64>,
) -> (Vec<PkAccountTup>, InstructionResult, u64) {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();

    let out_reserves = out_reserves.unwrap_or(amount * 2);
    let inp_reserves = inp_reserves.unwrap_or(amount * 2);

    let owner_transfer_amount = calculate_jupsol_wsol_inp_amount(
        amount,
        out_reserves,
        inp_reserves,
        fixture.out_lsd.lst_state.mint,
        fixture.inp_lsd.lst_state.mint,
    );

    let owner_accs = setup_owner_accounts(owner_transfer_amount);

    let instructions =
        build_rebalance_instruction_chain(&fixture, &owner_accs, amount, owner_transfer_amount);

    let accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    let accs_bef = accounts.clone();

    let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accounts));

    // Run StartRebalance ix separately to extract old_total_sol_value
    // from RebalanceRecord
    let start_result = SVM.with(|svm| svm.process_instruction(&instructions[0], &accs_bef));
    let rr_aft = start_result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID)
        .map(|(_, acc)| acc)
        .expect("rebalance record after start");
    let rebalance_record =
        unsafe { RebalanceRecord::of_acc_data(&rr_aft.data) }.expect("rebalance record");

    (accs_bef, result, rebalance_record.old_total_sol_value)
}

/// Validate that the transaction succeeded,
/// the pool state is not rebalancing before or after,
/// the pool did not lose SOL value,
/// the RebalanceRecord is properly closed,
/// and lamports are balanced.
fn assert_rebalance_transaction_success(
    accs_bef: &[PkAccountTup],
    result: &InstructionResult,
    old_total_sol_value: u64,
) {
    assert_eq!(result.program_result, ProgramResult::Success);

    let [pool_state_bef, pool_state_aft] = acc_bef_aft(
        &Pubkey::new_from_array(POOL_STATE_ID),
        accs_bef,
        &result.resulting_accounts,
    )
    .map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state()
    });

    assert_eq!(pool_state_bef.is_rebalancing, 0);
    assert_eq!(pool_state_aft.is_rebalancing, 0);
    assert!(pool_state_aft.total_sol_value >= old_total_sol_value);

    let rr_aft = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID);
    assert_eq!(rr_aft.unwrap().1.lamports, 0);

    assert_balanced(accs_bef, &result.resulting_accounts);
}

#[test]
fn rebalance_transaction_success() {
    let (accs_bef, result, old_total_sol_value) =
        execute_rebalance_transaction(100_000, None, None);

    assert_rebalance_transaction_success(&accs_bef, &result, old_total_sol_value);

    // Filter out executable accounts (programs and sysvars) which are not
    // relevant for rent-exemption checks - only user accounts matter
    let mut result_for_check = result.clone();
    result_for_check
        .resulting_accounts
        .retain(|(_, acc)| !acc.executable);

    // Assert all non-executable accounts are rent-exempt after transaction
    SVM.with(|svm| {
        assert!(result_for_check.run_checks(&[Check::all_rent_exempt()], &svm.config, svm));
    });
}

#[test]
fn missing_end_rebalance() {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();
    let (out_reserves, inp_reserves) = standard_reserves(100_000);

    let mut instructions = rebalance_ixs(
        &fixture.builder,
        fixture.out_idx,
        fixture.inp_idx,
        100_000,
        0,
        u64::MAX,
    );
    // Remove EndRebalance
    instructions.pop();

    let owner_accs = setup_owner_accounts(0);

    let accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    let result = SVM.with(|svm| svm.process_instruction(&instructions[0], &accounts));

    assert_jiminy_prog_err(
        &result.program_result,
        Inf1CtlCustomProgErr(NoSucceedingEndRebalance),
    );
}

#[test]
fn wrong_end_mint() {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();
    let (out_reserves, inp_reserves) = standard_reserves(100_000);

    let mut instructions = rebalance_ixs(
        &fixture.builder,
        fixture.out_idx,
        fixture.inp_idx,
        100_000,
        0,
        u64::MAX,
    );

    // Change EndRebalance instruction to use wrong inp_lst_mint
    if let Some(end_ix) = instructions.get_mut(1) {
        if end_ix.accounts.len() > END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT {
            end_ix.accounts[END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT].pubkey =
                Pubkey::new_unique();
        }
    }

    let owner_accs = setup_owner_accounts(0);

    let accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    let result = SVM.with(|svm| svm.process_instruction(&instructions[0], &accounts));

    assert_jiminy_prog_err(
        &result.program_result,
        Inf1CtlCustomProgErr(NoSucceedingEndRebalance),
    );
}

#[test]
fn no_transfer() {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();
    let (instructions, accounts) = setup_basic_rebalance_test(&fixture, 100_000, 0, u64::MAX);

    let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accounts));

    assert_jiminy_prog_err(
        &result.program_result,
        Inf1CtlCustomProgErr(PoolWouldLoseSolValue),
    );
}

#[test]
fn insufficient_transfer() {
    silence_mollusk_logs();

    let amount = 100_000;
    let fixture = setup_test_fixture();
    let (out_reserves, inp_reserves) = standard_reserves(amount);

    let required_amount = calculate_jupsol_wsol_inp_amount(
        amount,
        out_reserves,
        inp_reserves,
        fixture.out_lsd.lst_state.mint,
        fixture.inp_lsd.lst_state.mint,
    );

    let insufficient_amount = required_amount / 2;
    let owner_accs = setup_owner_accounts(insufficient_amount);

    let instructions =
        build_rebalance_instruction_chain(&fixture, &owner_accs, amount, insufficient_amount);

    let accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accounts));

    assert_jiminy_prog_err(
        &result.program_result,
        Inf1CtlCustomProgErr(PoolWouldLoseSolValue),
    );
}

#[test]
fn slippage_min_out_violated() {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();

    let (instructions, accounts) =
        setup_basic_rebalance_test(&fixture, 100_000, u64::MAX, u64::MAX);

    let result = SVM.with(|svm| svm.process_instruction(&instructions[0], &accounts));

    assert_jiminy_prog_err(
        &result.program_result,
        Inf1CtlCustomProgErr(SlippageToleranceExceeded),
    );
}

#[test]
fn slippage_max_inp_violated() {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();

    let (instructions, accounts) = setup_basic_rebalance_test(&fixture, 100_000, 0, 1);

    let result = SVM.with(|svm| svm.process_instruction(&instructions[0], &accounts));

    assert_jiminy_prog_err(
        &result.program_result,
        Inf1CtlCustomProgErr(SlippageToleranceExceeded),
    );
}

#[test]
fn multi_instruction_transfer_chain() {
    silence_mollusk_logs();

    let amount = 100_000;
    let fixture = setup_test_fixture();
    let (out_reserves, inp_reserves) = standard_reserves(amount);

    let total_transfer = calculate_jupsol_wsol_inp_amount(
        amount,
        out_reserves,
        inp_reserves,
        fixture.out_lsd.lst_state.mint,
        fixture.inp_lsd.lst_state.mint,
    );

    let owner_accs = setup_owner_accounts(total_transfer);

    let mut instructions = rebalance_ixs(
        &fixture.builder,
        fixture.out_idx,
        fixture.inp_idx,
        amount,
        0,
        u64::MAX,
    );

    let transfer1 = total_transfer / 3;
    let transfer2 = total_transfer / 3;
    let transfer3 = total_transfer - transfer1 - transfer2;

    let transfer_ix1 = create_transfer_ix(
        owner_accs.owner,
        owner_accs.owner_token_account,
        fixture.inp_lsd.pool_reserves,
        transfer1,
    );
    let transfer_ix2 = create_transfer_ix(
        owner_accs.owner,
        owner_accs.owner_token_account,
        fixture.inp_lsd.pool_reserves,
        transfer2,
    );
    let transfer_ix3 = create_transfer_ix(
        owner_accs.owner,
        owner_accs.owner_token_account,
        fixture.inp_lsd.pool_reserves,
        transfer3,
    );

    instructions.insert(1, transfer_ix1);
    instructions.insert(2, transfer_ix2);
    instructions.insert(3, transfer_ix3);

    let accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    let accs_bef = accounts.clone();
    let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accounts));

    let start_result = SVM.with(|svm| svm.process_instruction(&instructions[0], &accs_bef));
    let rr_aft = start_result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID)
        .map(|(_, acc)| acc)
        .expect("rebalance record after start");
    let rebalance_record =
        unsafe { RebalanceRecord::of_acc_data(&rr_aft.data) }.expect("rebalance record");

    assert_rebalance_transaction_success(&accs_bef, &result, rebalance_record.old_total_sol_value);
}

#[test]
fn rebalance_chain_updates_reserves_correctly() {
    silence_mollusk_logs();

    let amount = 100_000;
    let fixture = setup_test_fixture();
    let (out_reserves, inp_reserves) = standard_reserves(amount);

    let transfer_amount = calculate_jupsol_wsol_inp_amount(
        amount,
        out_reserves,
        inp_reserves,
        fixture.out_lsd.lst_state.mint,
        fixture.inp_lsd.lst_state.mint,
    );

    let owner_accs = setup_owner_accounts(transfer_amount);

    let instructions =
        build_rebalance_instruction_chain(&fixture, &owner_accs, amount, transfer_amount);

    let accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    let accs_bef = accounts.clone();

    let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accounts));

    assert_eq!(result.program_result, ProgramResult::Success);

    let [out_reserves_bef, out_reserves_aft] = acc_bef_aft(
        &Pubkey::new_from_array(fixture.out_lsd.pool_reserves),
        &accs_bef,
        &result.resulting_accounts,
    )
    .map(|a| get_token_account_amount(&a.data));

    let [inp_reserves_bef, inp_reserves_aft] = acc_bef_aft(
        &Pubkey::new_from_array(fixture.inp_lsd.pool_reserves),
        &accs_bef,
        &result.resulting_accounts,
    )
    .map(|a| get_token_account_amount(&a.data));

    let [withdraw_to_bef, withdraw_to_aft] = acc_bef_aft(
        &Pubkey::new_from_array(fixture.withdraw_to),
        &accs_bef,
        &result.resulting_accounts,
    )
    .map(|a| get_token_account_amount(&a.data));

    assert_eq!(
        out_reserves_aft,
        out_reserves_bef - amount,
        "out reserves should decrease by withdrawal amount"
    );
    assert_eq!(
        inp_reserves_aft,
        inp_reserves_bef + transfer_amount,
        "inp reserves should increase by transfer amount"
    );
    assert_eq!(
        withdraw_to_aft,
        withdraw_to_bef + amount,
        "withdraw_to should receive withdrawn LST"
    );

    assert_balanced(&accs_bef, &result.resulting_accounts);
}

#[test]
fn rebalance_record_lifecycle() {
    silence_mollusk_logs();

    let amount = 100_000;

    let (accs_bef, result, old_total_sol_value) = execute_rebalance_transaction(amount, None, None);

    assert_eq!(result.program_result, ProgramResult::Success);

    let [pool_state_bef, pool_state_aft] = acc_bef_aft(
        &Pubkey::new_from_array(POOL_STATE_ID),
        &accs_bef,
        &result.resulting_accounts,
    )
    .map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .expect("pool state")
            .into_pool_state()
    });

    assert_eq!(pool_state_bef.is_rebalancing, 0);

    let rr_bef = accs_bef
        .iter()
        .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID);
    assert_eq!(
        rr_bef.map(|(_, acc)| acc.lamports).unwrap(),
        0,
        "rebalance record should not exist initially"
    );

    assert_eq!(pool_state_bef.is_rebalancing, 0);
    assert_eq!(pool_state_aft.is_rebalancing, 0);

    assert!(pool_state_aft.total_sol_value >= old_total_sol_value);

    let rr_aft = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID);
    assert_eq!(rr_aft.unwrap().1.lamports, 0);

    // Verify RebalanceRecord creation by executing just StartRebalance
    let fixture2 = setup_test_fixture();
    let (start_ixs, start_accounts) = setup_basic_rebalance_test(&fixture2, amount, 0, u64::MAX);

    let start_result = SVM.with(|svm| svm.process_instruction(&start_ixs[0], &start_accounts));
    assert_eq!(start_result.program_result, ProgramResult::Success);

    let [pool_state_bef, pool_state_aft] = acc_bef_aft(
        &Pubkey::new_from_array(POOL_STATE_ID),
        &start_accounts,
        &start_result.resulting_accounts,
    )
    .map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .expect("pool state")
            .into_pool_state()
    });

    assert_eq!(pool_state_bef.is_rebalancing, 0);
    assert_eq!(pool_state_aft.is_rebalancing, 1);

    let rr_aft = start_result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID)
        .map(|(_, acc)| acc)
        .expect("rebalance record after start");

    assert!(rr_aft.lamports > 0);

    let rebalance_record =
        unsafe { RebalanceRecord::of_acc_data(&rr_aft.data) }.expect("rebalance record");

    assert_eq!(rebalance_record.inp_lst_index, fixture2.inp_idx);

    assert!(rebalance_record.old_total_sol_value > 0);

    assert_balanced(&accs_bef, &result.resulting_accounts);
}

#[test]
fn pool_already_rebalancing() {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();
    let owner_accs = setup_owner_accounts(0);
    let (out_reserves, inp_reserves) = standard_reserves(100_000);

    let first_instructions = rebalance_ixs(
        &fixture.builder,
        fixture.out_idx,
        fixture.inp_idx,
        100_000,
        0,
        u64::MAX,
    );

    let accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &first_instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    // Execute first StartRebalance instruction to set pool.is_rebalancing = 1
    let result = SVM.with(|svm| svm.process_instruction(&first_instructions[0], &accounts));
    assert_eq!(result.program_result, ProgramResult::Success);

    let pool_state_aft = result
        .resulting_accounts
        .iter()
        .find(|(pk, _)| pk.to_bytes() == POOL_STATE_ID)
        .map(|(_, acc)| {
            PoolStatePacked::of_acc_data(&acc.data)
                .expect("pool state")
                .into_pool_state()
        })
        .expect("pool state");
    assert_eq!(pool_state_aft.is_rebalancing, 1);

    let second_instructions = rebalance_ixs(
        &fixture.builder,
        fixture.out_idx,
        fixture.inp_idx,
        100_000,
        0,
        u64::MAX,
    );

    let mut accounts_with_second_ix = result.resulting_accounts.clone();
    upsert_account(
        &mut accounts_with_second_ix,
        (
            Pubkey::new_from_array(INSTRUCTIONS_SYSVAR_ID),
            mock_instructions_sysvar(&second_instructions, 0),
        ),
    );

    // Execute another StartRebalance instruction
    let result2 =
        SVM.with(|svm| svm.process_instruction(&second_instructions[0], &accounts_with_second_ix));

    assert_jiminy_prog_err(
        &result2.program_result,
        Inf1CtlCustomProgErr(PoolRebalancing),
    );
}

#[test]
fn unauthorized_rebalance_authority() {
    silence_mollusk_logs();

    let fixture = setup_test_fixture();
    let owner_accs = setup_owner_accounts(0);
    let (out_reserves, inp_reserves) = standard_reserves(100_000);

    let unauthorized_pk = Pubkey::new_unique().to_bytes();
    let unauthorized_builder = jupsol_wsol_builder(
        unauthorized_pk,
        fixture.out_lsd.lst_state.mint,
        fixture.inp_lsd.lst_state.mint,
        fixture.withdraw_to,
    );

    let instructions = rebalance_ixs(
        &unauthorized_builder,
        fixture.out_idx,
        fixture.inp_idx,
        100_000,
        0,
        u64::MAX,
    );

    let mut accounts = setup_rebalance_transaction_accounts(
        &fixture,
        &instructions,
        out_reserves,
        inp_reserves,
        &owner_accs,
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(unauthorized_pk),
            mock_sys_acc(100_000_000_000),
        ),
    );

    let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accounts));

    assert_jiminy_prog_err(&result.program_result, INVALID_ARGUMENT);
}

proptest! {
  #[test]
  fn rebalance_transaction_various_amounts_any(
      amount in 1u64..=1_000_000_000,
      out_reserve_multiplier in 2u64..=100,
      inp_reserve_multiplier in 2u64..=100,
  ) {
      let out_reserves = amount.saturating_mul(out_reserve_multiplier);
      let inp_reserves = amount.saturating_mul(inp_reserve_multiplier);

      let (accs_bef, result, old_total_sol_value) =
          execute_rebalance_transaction(amount, Some(out_reserves), Some(inp_reserves));

      assert_rebalance_transaction_success(&accs_bef, &result, old_total_sol_value);
  }
}

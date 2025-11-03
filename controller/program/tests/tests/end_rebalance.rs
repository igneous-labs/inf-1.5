use crate::{
    common::{jupsol_fixtures_svc_suf, MAX_LST_STATES, SVM},
    utils::rebalance::{
        assert_start_success, fixture_pool_and_lsl, instructions_sysvar,
        mock_empty_rebalance_record_account, rebalance_ixs, start_rebalance_ix_pre_keys_owned,
        StartRebalanceKeysBuilder,
    },
};

use expect_test::expect;

use inf1_core::instructions::rebalance::end::EndRebalanceIxAccs;

use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::{PoolState, PoolStatePacked},
        rebalance_record::RebalanceRecord,
    },
    err::Inf1CtlErr,
    instructions::rebalance::end::{
        EndRebalanceIxData, EndRebalanceIxPreKeysOwned, NewEndRebalanceIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::U8BoolMut,
    ID,
};

use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAgTy,
};

use inf1_test_utils::{
    acc_bef_aft, assert_diffs_pool_state, find_pool_reserves_ata, keys_signer_writable_to_metas,
    lst_state_list_account, mock_prog_acc, mock_system_program_account, mock_token_acc,
    pool_state_account, raw_token_acc, silence_mollusk_logs, upsert_account, Diff,
    DiffsPoolStateArgs, NewPoolStateBoolsBuilder, PkAccountTup, ALL_FIXTURES,
    JUPSOL_FIXTURE_LST_IDX,
};

use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, NOT_ENOUGH_ACCOUNT_KEYS};

use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use proptest::test_runner::TestCaseResult;
use sanctum_system_jiminy::sanctum_system_core::ID as SYSTEM_PROGRAM_ID;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

struct StartContext {
    rebalance_auth: [u8; 32],
    inp_mint: [u8; 32],
    pool_total_sol_value: u64,
    amount: u64,
    inp_reserves: Pubkey,
}

fn run_start_fixture(amount: u64) -> (Vec<PkAccountTup>, StartContext) {
    let (pool, mut lst_state_bytes) = fixture_pool_and_lsl();
    let packed_list = LstStatePackedList::of_acc_data(&lst_state_bytes).expect("lst packed");
    let out_mint = packed_list.0[JUPSOL_FIXTURE_LST_IDX].into_lst_state().mint;
    let (inp_idx, inp_state) = packed_list
        .0
        .iter()
        .enumerate()
        .find(|(_, state)| state.into_lst_state().mint != out_mint)
        .expect("second lst present");
    let inp_mint = inp_state.into_lst_state().mint;

    let withdraw_to = Pubkey::new_unique().to_bytes();
    let rebalance_auth = pool.rebalance_authority;

    let start_ix_prefix = start_rebalance_ix_pre_keys_owned(
        rebalance_auth,
        &TOKENKEG_PROGRAM,
        out_mint,
        inp_mint,
        withdraw_to,
    );
    let start_builder = StartRebalanceKeysBuilder {
        ix_prefix: start_ix_prefix,
        out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        out_calc: jupsol_fixtures_svc_suf(),
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    };

    {
        let list_mut = LstStatePackedListMut::of_acc_data(&mut lst_state_bytes).unwrap();
        if let Some(packed) = list_mut.0.get_mut(JUPSOL_FIXTURE_LST_IDX) {
            unsafe {
                packed.as_lst_state_mut().sol_value_calculator = start_builder.out_calc_prog;
            }
        }
        if let Some(packed) = list_mut.0.get_mut(inp_idx) {
            unsafe {
                packed.as_lst_state_mut().sol_value_calculator = start_builder.inp_calc_prog;
            }
        }
    }

    let ixs = rebalance_ixs(
        &start_builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        inp_idx as u32,
        amount,
        0,
        u64::MAX,
    );

    let mut accounts: Vec<PkAccountTup> = ALL_FIXTURES
        .iter()
        .map(|(pk, acc)| (*pk, acc.clone()))
        .collect();
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(POOL_STATE_ID),
            pool_state_account(pool),
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(LST_STATE_LIST_ID),
            lst_state_list_account(lst_state_bytes.clone()),
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(rebalance_auth),
            Account {
                lamports: u64::MAX,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Account::default()
            },
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(withdraw_to),
            mock_token_acc(raw_token_acc(out_mint, withdraw_to, 0)),
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(REBALANCE_RECORD_ID),
            mock_empty_rebalance_record_account(),
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
            mock_system_program_account(),
        ),
    );
    upsert_account(&mut accounts, instructions_sysvar(&ixs, 0));
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(*SvcAgTy::Wsol(()).svc_program_id()),
            mock_prog_acc(Pubkey::new_unique()),
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(*SvcAgTy::SanctumSplMulti(()).svc_program_id()),
            mock_prog_acc(Pubkey::new_unique()),
        ),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ixs[0], &accounts));
    assert_eq!(
        program_result,
        ProgramResult::Success,
        "StartRebalance failed"
    );
    assert_start_success(&accounts, &resulting_accounts, &out_mint, &inp_mint);

    let (inp_reserves_pk, _) = find_pool_reserves_ata(&TOKENKEG_PROGRAM, &inp_mint);

    (
        resulting_accounts,
        StartContext {
            rebalance_auth,
            inp_mint,
            pool_total_sol_value: pool.total_sol_value,
            amount,
            inp_reserves: inp_reserves_pk,
        },
    )
}

fn process_end_instruction(
    instruction: &Instruction,
    accounts: &[PkAccountTup],
) -> InstructionResult {
    SVM.with(|svm| svm.process_instruction(instruction, accounts))
}

struct EndCaseOutcome {
    program_result: ProgramResult,
    resulting_accounts: Vec<PkAccountTup>,
    initial_accounts: Vec<PkAccountTup>,
    pool_total_sol_value_before: u64,
}

fn execute_end_case(
    amount: u64,
    pool_sol_value_delta: i64,
    pool_not_rebalancing: bool,
    rebalance_auth_override: Option<[u8; 32]>,
    inp_mint_override: Option<[u8; 32]>,
    additional_accounts: impl IntoIterator<Item = PkAccountTup>,
) -> EndCaseOutcome {
    let (mut accounts, ctx) = run_start_fixture(amount);

    credit_token_amount(&mut accounts, ctx.inp_reserves, ctx.amount);

    if let Some(pool_acc) = find_account_mut(&mut accounts, &Pubkey::new_from_array(POOL_STATE_ID))
    {
        if let Some(pool) = unsafe { PoolState::of_acc_data_mut(&mut pool_acc.data) } {
            // Simulates a scenario where the pool loses value during rebalance
            if pool_sol_value_delta < 0 {
                pool.total_sol_value = pool
                    .total_sol_value
                    .saturating_sub(pool_sol_value_delta.unsigned_abs());
            } else {
                // Simulates a scenario where the pool gains value during rebalance
                pool.total_sol_value = pool.total_sol_value.saturating_add(ctx.amount);
                if pool_sol_value_delta > 0 {
                    pool.total_sol_value = pool
                        .total_sol_value
                        .saturating_add(pool_sol_value_delta as u64);
                }
            }

            if pool_not_rebalancing {
                U8BoolMut(&mut pool.is_rebalancing).set_false();
            }
        }
    }

    let rebalance_auth = rebalance_auth_override.unwrap_or(ctx.rebalance_auth);
    let inp_mint = inp_mint_override.unwrap_or(ctx.inp_mint);

    let ix_prefix = end_rebalance_ix_pre_keys_owned(rebalance_auth, inp_mint);

    let builder = EndRebalanceKeysBuilder {
        ix_prefix,
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    };

    let instruction = end_rebalance_ix(&builder);

    additional_accounts
        .into_iter()
        .for_each(|account| upsert_account(&mut accounts, account));

    let initial_accounts = accounts.clone();
    let result = process_end_instruction(&instruction, &accounts);

    EndCaseOutcome {
        program_result: result.program_result,
        resulting_accounts: result.resulting_accounts,
        initial_accounts,
        pool_total_sol_value_before: ctx.pool_total_sol_value,
    }
}

fn run_end_case(
    amount: u64,
    pool_sol_value_delta: i64,
    pool_not_rebalancing: bool,
    rebalance_auth_override: Option<[u8; 32]>,
    inp_mint_override: Option<[u8; 32]>,
    additional_accounts: impl IntoIterator<Item = PkAccountTup>,
    error_type: Option<EndError>,
) -> TestCaseResult {
    let outcome = execute_end_case(
        amount,
        pool_sol_value_delta,
        pool_not_rebalancing,
        rebalance_auth_override,
        inp_mint_override,
        additional_accounts,
    );

    if let Some(error_type) = error_type {
        let expected = end_error_to_program_error(error_type);
        inf1_test_utils::assert_jiminy_prog_err(&outcome.program_result, expected);
    } else {
        prop_assert_eq!(outcome.program_result, ProgramResult::Success);
        assert_end_success(
            &outcome.initial_accounts,
            &outcome.resulting_accounts,
            outcome.pool_total_sol_value_before,
        );
    }

    Ok(())
}

proptest! {
    #[test]
    fn end_rebalance_unauthorized_any(amount in 50_000u64..=400_000u64) {
        let wrong_auth = Pubkey::new_unique().to_bytes();
        let wrong_auth_acc = Account {
            lamports: u64::MAX,
            owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
            ..Account::default()
        };
        run_end_case(
            amount,
            0,
            false,
            Some(wrong_auth),
            None,
            [(Pubkey::new_from_array(wrong_auth), wrong_auth_acc)],
            Some(EndError::Unauthorized),
        )?;
    }
}

proptest! {
    #[test]
    fn end_rebalance_pool_not_rebalancing_any(amount in 50_000u64..=400_000u64) {
        run_end_case(
            amount,
            0,
            true,
            None,
            None,
            [],
            Some(EndError::PoolNotRebalancing),
        )?;
    }
}

proptest! {
    #[test]
    fn end_rebalance_invalid_data_size_any(amount in 50_000u64..=400_000u64) {
        let mut truncated_record = mock_empty_rebalance_record_account();
        truncated_record.data.truncate(4);
        run_end_case(
            amount,
            0,
            false,
            None,
            None,
            [(Pubkey::new_from_array(REBALANCE_RECORD_ID), truncated_record)],
            Some(EndError::InvalidRebalanceRecordData),
        )?;
    }
}

proptest! {
    #[test]
    fn end_rebalance_invalid_dst_index_any(amount in 50_000u64..=400_000u64) {
        let mut invalid_idx_record = mock_empty_rebalance_record_account();
        if let Some(rec) = unsafe { RebalanceRecord::of_acc_data_mut(&mut invalid_idx_record.data) } {
            rec.dst_lst_index = (MAX_LST_STATES as u32) + 10;
        }
        run_end_case(
            amount,
            0,
            false,
            None,
            None,
            [(Pubkey::new_from_array(REBALANCE_RECORD_ID), invalid_idx_record)],
            Some(EndError::InvalidLstIndex),
        )?;
    }
}

proptest! {
    #[test]
    fn end_rebalance_pool_would_lose_sol_value_any(amount in 50_000u64..=400_000u64) {
        run_end_case(
            amount,
            -(amount as i64),
            false,
            None,
            None,
            [],
            Some(EndError::PoolWouldLoseSolValue),
        )?;
    }
}

proptest! {
    #[test]
    fn end_rebalance_success_any(amount in 50_000u64..=400_000u64) {
        run_end_case(
            amount,
            0,
            false,
            None,
            None,
            [],
            None
        )?;
    }
}

#[test]
fn end_rebalance_donation_success() {
    let outcome = execute_end_case(100_000, 50_000, false, None, None, []);
    assert_eq!(outcome.program_result, ProgramResult::Success);
    assert_end_success(
        &outcome.initial_accounts,
        &outcome.resulting_accounts,
        outcome.pool_total_sol_value_before,
    );
}

#[test]
fn end_rebalance_missing_calc_accounts_fails() {
    let amount = 100_000;
    let (mut accounts, ctx) = run_start_fixture(amount);

    credit_token_amount(&mut accounts, ctx.inp_reserves, ctx.amount);

    if let Some(pool_acc) = find_account_mut(&mut accounts, &Pubkey::new_from_array(POOL_STATE_ID))
    {
        if let Some(pool) = unsafe { PoolState::of_acc_data_mut(&mut pool_acc.data) } {
            pool.total_sol_value = pool.total_sol_value.saturating_add(ctx.amount);
        }
    }

    let ix_prefix = end_rebalance_ix_pre_keys_owned(ctx.rebalance_auth, ctx.inp_mint);
    let builder = EndRebalanceKeysBuilder {
        ix_prefix,
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    };
    let mut instruction = end_rebalance_ix(&builder);

    instruction.accounts.truncate(6);

    let result = process_end_instruction(&instruction, &accounts);
    inf1_test_utils::assert_jiminy_prog_err(&result.program_result, NOT_ENOUGH_ACCOUNT_KEYS);
}

#[test]
fn end_rebalance_jupsol_fixture_snapshot() {
    silence_mollusk_logs();

    let amount = 250_000;
    let (accounts_after_start, _ctx) = run_start_fixture(amount);
    let outcome = execute_end_case(amount, 0, false, None, None, []);

    assert_eq!(outcome.program_result, ProgramResult::Success);

    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_bef = accounts_after_start
        .iter()
        .find(|(pk, _)| *pk == pool_pk)
        .map(|(_, acc)| acc)
        .expect("pool account before");
    let pool_aft = outcome
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == pool_pk)
        .map(|(_, acc)| acc)
        .expect("pool account after");

    let pool_state_bef = PoolStatePacked::of_acc_data(&pool_bef.data)
        .expect("pool before")
        .into_pool_state();
    let pool_state_aft = PoolStatePacked::of_acc_data(&pool_aft.data)
        .expect("pool after")
        .into_pool_state();

    let is_rebalancing_bef_str = format!("{}", pool_state_bef.is_rebalancing);
    expect!["1"].assert_eq(&is_rebalancing_bef_str);

    let is_rebalancing_aft_str = format!("{}", pool_state_aft.is_rebalancing);
    expect!["0"].assert_eq(&is_rebalancing_aft_str);

    assert!(
        pool_state_aft.total_sol_value >= pool_state_bef.total_sol_value,
        "pool should maintain or increase SOL value"
    );

    let rebalance_record_pk = Pubkey::new_from_array(REBALANCE_RECORD_ID);
    let rebalance_record_aft = outcome
        .resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == rebalance_record_pk)
        .map(|(_, acc)| acc)
        .expect("rebalance record account");

    let lamports_str = format!("{}", rebalance_record_aft.lamports);
    expect!["0"].assert_eq(&lamports_str);
}

pub type EndRebalanceKeysBuilder =
    EndRebalanceIxAccs<[u8; 32], EndRebalanceIxPreKeysOwned, SvcCalcAccsAg>;

pub fn assert_end_success(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    expected_old_total_sol_value: u64,
) {
    let [pool_bef_acc, pool_aft_acc] =
        acc_bef_aft(&Pubkey::new_from_array(POOL_STATE_ID), bef, aft);

    let pool_bef = PoolStatePacked::of_acc_data(&pool_bef_acc.data)
        .expect("pool before")
        .into_pool_state();
    let pool_aft = PoolStatePacked::of_acc_data(&pool_aft_acc.data)
        .expect("pool after")
        .into_pool_state();

    assert_diffs_pool_state(
        &DiffsPoolStateArgs {
            bools: NewPoolStateBoolsBuilder::start()
                .with_is_rebalancing(Diff::StrictChanged(true, false))
                .with_is_disabled(Diff::Pass)
                .build(),
            total_sol_value: Diff::GreaterOrEqual(expected_old_total_sol_value),
            ..Default::default()
        },
        &pool_bef,
        &pool_aft,
    );

    let rr_pk = Pubkey::new_from_array(REBALANCE_RECORD_ID);
    let rr_after = aft.iter().find(|(pk, _)| *pk == rr_pk);
    assert!(
        rr_after.is_none() || rr_after.unwrap().1.lamports == 0,
        "rebalance record should be closed after EndRebalance"
    );
}

pub fn end_rebalance_ix_pre_keys_owned(
    rebalance_auth: [u8; 32],
    inp_mint: [u8; 32],
) -> EndRebalanceIxPreKeysOwned {
    let rebalance_record_pda = Pubkey::new_from_array(REBALANCE_RECORD_ID);

    NewEndRebalanceIxPreAccsBuilder::start()
        .with_rebalance_auth(rebalance_auth)
        .with_pool_state(POOL_STATE_ID)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_rebalance_record(rebalance_record_pda.to_bytes())
        .with_inp_lst_mint(inp_mint)
        .with_inp_pool_reserves(
            inf1_test_utils::find_pool_reserves_ata(&TOKENKEG_PROGRAM, &inp_mint)
                .0
                .to_bytes(),
        )
        .build()
}

pub fn end_rebalance_ix(builder: &EndRebalanceKeysBuilder) -> Instruction {
    let keys_owned = builder.keys_owned();
    let accounts = keys_signer_writable_to_metas(
        keys_owned.seq(),
        builder.is_signer().seq(),
        builder.is_writer().seq(),
    );

    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: EndRebalanceIxData::new().as_buf().into(),
    }
}

pub fn credit_token_amount(accounts: &mut [PkAccountTup], key: Pubkey, delta: u64) {
    if let Some((_, acc)) = accounts.iter_mut().find(|(pk, _)| *pk == key) {
        if acc.data.len() >= 72 {
            let mut amount_bytes = [0u8; 8];
            amount_bytes.copy_from_slice(&acc.data[64..72]);
            let current = u64::from_le_bytes(amount_bytes);
            let new_amount = current.saturating_add(delta);
            acc.data[64..72].copy_from_slice(&new_amount.to_le_bytes());
        }
    }
}

pub fn find_account_mut<'a>(
    accounts: &'a mut [PkAccountTup],
    key: &Pubkey,
) -> Option<&'a mut Account> {
    accounts
        .iter_mut()
        .find_map(|(pk, acc)| (*pk == *key).then_some(acc))
}

#[derive(Clone, Copy, Debug)]
pub enum EndError {
    Unauthorized,
    PoolNotRebalancing,
    InvalidLstIndex,
    InvalidRebalanceRecordData,
    PoolWouldLoseSolValue,
}

pub fn end_error_to_program_error(err: EndError) -> ProgramError {
    match err {
        EndError::Unauthorized => INVALID_ARGUMENT.into(),
        EndError::PoolNotRebalancing => Inf1CtlCustomProgErr(Inf1CtlErr::PoolNotRebalancing).into(),
        EndError::InvalidLstIndex => Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex).into(),
        EndError::InvalidRebalanceRecordData => {
            Inf1CtlCustomProgErr(Inf1CtlErr::InvalidRebalanceRecordData).into()
        }
        EndError::PoolWouldLoseSolValue => {
            Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into()
        }
    }
}

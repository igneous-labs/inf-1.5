use crate::{
    common::{jupsol_fixtures_svc_suf, MAX_LST_STATES, SVM},
    utils::rebalance::{
        assert_start_success, fixture_pool_and_lsl, instructions_sysvar,
        mock_empty_rebalance_record_account, rebalance_ixs, start_rebalance_ix_pre_keys_owned,
        StartRebalanceKeysBuilder,
    },
};

use expect_test::expect;

use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::{PoolState, PoolStatePacked},
        rebalance_record::{RebalanceRecord, RebalanceRecordPacked},
    },
    err::Inf1CtlErr,
    instructions::rebalance::{
        end::END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT,
        start::{StartRebalanceIxArgs, StartRebalanceIxData},
    },
    keys::{INSTRUCTIONS_SYSVAR_ID, LST_STATE_LIST_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
    program_err::Inf1CtlCustomProgErr,
    typedefs::{lst_state::LstState, u8bool::U8BoolMut},
};

use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAgTy,
};

use inf1_svc_jiminy::traits::SolValCalcAccs;

use inf1_test_utils::{
    any_lst_state, any_lst_state_list, any_normal_pk, any_pool_state, create_pool_reserves_ata,
    create_protocol_fee_accumulator_ata, fixtures_accounts_opt_cloned, lst_state_list_account,
    mock_mint, mock_prog_acc, mock_token_acc, pool_state_account, raw_mint, raw_token_acc,
    silence_mollusk_logs, upsert_account, AnyLstStateArgs, AnyPoolStateArgs, LstStateData,
    LstStateListData, PkAccountTup, PoolStateBools, JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT, WSOL_MINT,
};

use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, NOT_ENOUGH_ACCOUNT_KEYS};

use mollusk_svm::{
    program::keyed_account_for_system_program,
    result::{InstructionResult, ProgramResult},
};

use proptest::{prelude::*, test_runner::TestCaseResult};

use sanctum_system_jiminy::sanctum_system_core::ID as SYSTEM_PROGRAM_ID;

use solana_account::Account;
use solana_pubkey::Pubkey;

use std::collections::HashMap;

fn compute_lst_indices(
    lsl: &mut LstStateListData,
    out_lsd: LstStateData,
    inp_lsd: LstStateData,
) -> (u32, u32) {
    let out_idx = lsl.upsert(out_lsd) as u32;
    let inp_idx = lsl.upsert(inp_lsd) as u32;
    (out_idx, inp_idx)
}

fn wsol_builder(
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
) -> StartRebalanceKeysBuilder {
    let ix_prefix = start_rebalance_ix_pre_keys_owned(
        rebalance_auth,
        &TOKENKEG_PROGRAM,
        out_mint,
        inp_mint,
        withdraw_to,
    );

    StartRebalanceKeysBuilder {
        ix_prefix,
        out_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        out_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    }
}

struct JupSolFixtureData {
    pool: PoolState,
    lst_state_bytes: Vec<u8>,
    lst_states: Vec<LstState>,
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    inp_idx: usize,
}

fn load_jupsol_fixture() -> JupSolFixtureData {
    let (pool, lst_state_bytes) = fixture_pool_and_lsl();
    let rebalance_auth = pool.rebalance_authority;
    let out_mint = JUPSOL_MINT.to_bytes();

    let lst_states = LstStatePackedList::of_acc_data(&lst_state_bytes)
        .expect("lst packed")
        .0
        .iter()
        .map(|x| x.into_lst_state())
        .collect::<Vec<_>>();
    let inp_mint = lst_states
        .iter()
        .find(|state| state.mint != out_mint)
        .expect("second lst")
        .mint;
    let inp_idx = lst_states
        .iter()
        .position(|state| state.mint == inp_mint)
        .unwrap();

    JupSolFixtureData {
        pool,
        lst_state_bytes,
        lst_states,
        rebalance_auth,
        out_mint,
        inp_mint,
        inp_idx,
    }
}

fn jupsol_wsol_builder(
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
) -> StartRebalanceKeysBuilder {
    let ix_prefix = start_rebalance_ix_pre_keys_owned(
        rebalance_auth,
        &TOKENKEG_PROGRAM,
        out_mint,
        inp_mint,
        withdraw_to,
    );

    StartRebalanceKeysBuilder {
        ix_prefix,
        out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        out_calc: jupsol_fixtures_svc_suf(),
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    }
}

fn default_ix_args(out_idx: u32, inp_idx: u32, amount: u64) -> StartRebalanceIxArgs {
    StartRebalanceIxArgs {
        out_lst_value_calc_accs: 1,
        out_lst_index: out_idx,
        inp_lst_index: inp_idx,
        amount,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
    }
}

fn start_rebalance_fixtures_accounts(builder: &StartRebalanceKeysBuilder) -> Vec<PkAccountTup> {
    let keys_owned = builder.keys_owned();
    fixtures_accounts_opt_cloned(keys_owned.seq().copied()).collect()
}

struct StartCaseOutcome {
    program_result: ProgramResult,
    resulting_accounts: Vec<PkAccountTup>,
    initial_accounts: Vec<PkAccountTup>,
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
}

#[allow(clippy::too_many_arguments)]
fn execute_start_case(
    pool: PoolState,
    lsl: LstStateListData,
    out_lsd: LstStateData,
    inp_lsd: LstStateData,
    builder: StartRebalanceKeysBuilder,
    ix_args: StartRebalanceIxArgs,
    out_balance: u64,
    inp_balance: u64,
    withdraw_to: [u8; 32],
    include_end_rebalance: bool,
    ix_data_override: Option<StartRebalanceIxArgs>,
    additional_accounts: impl IntoIterator<Item = PkAccountTup>,
) -> StartCaseOutcome {
    silence_mollusk_logs();

    let LstStateListData {
        mut lst_state_list,
        all_pool_reserves,
        ..
    } = lsl;

    let out_lst_idx = ix_args.out_lst_index as usize;
    let inp_lst_idx = ix_args.inp_lst_index as usize;

    {
        let list_mut = LstStatePackedListMut::of_acc_data(&mut lst_state_list).unwrap();
        if let Some(packed) = list_mut.0.get_mut(out_lst_idx) {
            unsafe {
                U8BoolMut(&mut packed.as_lst_state_mut().is_input_disabled).set_false();
            };
        }
        if let Some(packed) = list_mut.0.get_mut(inp_lst_idx) {
            unsafe {
                packed.as_lst_state_mut().is_input_disabled = inp_lsd.lst_state.is_input_disabled;
            };
        }
    }

    let out_mint = out_lsd.lst_state.mint;
    let inp_mint = inp_lsd.lst_state.mint;

    let actual_out_balance = if out_mint == inp_mint {
        inp_balance
    } else {
        out_balance
    };

    let mut instructions = rebalance_ixs(
        &builder,
        ix_args.out_lst_index,
        ix_args.inp_lst_index,
        ix_args.amount,
        ix_args.min_starting_out_lst,
        ix_args.max_starting_inp_lst,
    );

    if !include_end_rebalance {
        instructions.pop();
    }

    // Override out_lst_value_calc_accs by modifying ix data
    if let Some(override_args) = ix_data_override {
        if let Some(start_ix) = instructions.first_mut() {
            start_ix.data = StartRebalanceIxData::new(override_args).as_buf().into();
        }
    }

    let mut accounts = start_rebalance_fixtures_accounts(&builder);

    let rebalance_auth = *builder.ix_prefix.rebalance_auth();

    add_common_accounts(
        &mut accounts,
        &pool,
        &lst_state_list,
        Some(&all_pool_reserves),
        rebalance_auth,
        out_mint,
        inp_mint,
        withdraw_to,
        actual_out_balance,
        inp_balance,
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(rebalance_auth),
            Account {
                lamports: u64::MAX,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Default::default()
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
            Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
            keyed_account_for_system_program().1,
        ),
    );

    {
        let out_prog = Pubkey::new_from_array(builder.out_calc_prog);
        let calc_acc = mock_prog_acc(Pubkey::new_unique());
        upsert_account(&mut accounts, (out_prog, calc_acc));
    }

    {
        let inp_prog = Pubkey::new_from_array(builder.inp_calc_prog);
        let calc_acc = mock_prog_acc(Pubkey::new_unique());
        upsert_account(&mut accounts, (inp_prog, calc_acc));
    }

    additional_accounts
        .into_iter()
        .for_each(|account| upsert_account(&mut accounts, account));

    let instructions_account = instructions_sysvar(&instructions, 0).1;
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(INSTRUCTIONS_SYSVAR_ID),
            instructions_account,
        ),
    );

    let rebalance_record_pda = Pubkey::new_from_array(REBALANCE_RECORD_ID);
    upsert_account(
        &mut accounts,
        (rebalance_record_pda, mock_empty_rebalance_record_account()),
    );

    let initial_accounts = accounts.clone();
    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&instructions[0], &accounts));

    StartCaseOutcome {
        program_result,
        resulting_accounts,
        initial_accounts,
        out_mint,
        inp_mint,
    }
}

#[allow(clippy::too_many_arguments)]
fn run_start_case(
    pool: PoolState,
    lsl: LstStateListData,
    out_lsd: LstStateData,
    inp_lsd: LstStateData,
    builder: StartRebalanceKeysBuilder,
    ix_args: StartRebalanceIxArgs,
    out_balance: u64,
    inp_balance: u64,
    withdraw_to: [u8; 32],
    include_end_rebalance: bool,
    ix_data_override: Option<StartRebalanceIxArgs>,
    additional_accounts: impl IntoIterator<Item = PkAccountTup>,
    expected_err: Option<impl Into<ProgramError>>,
) -> TestCaseResult {
    let outcome = execute_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        ix_args,
        out_balance,
        inp_balance,
        withdraw_to,
        include_end_rebalance,
        ix_data_override,
        additional_accounts,
    );

    if let Some(expected) = expected_err {
        inf1_test_utils::assert_jiminy_prog_err(&outcome.program_result, expected);
    } else {
        prop_assert_eq!(outcome.program_result, ProgramResult::Success);
        assert_start_success(
            &outcome.initial_accounts,
            &outcome.resulting_accounts,
            &outcome.out_mint,
            &outcome.inp_mint,
        );
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_wsol_proptest_case(
    pool: PoolState,
    mut lsl: LstStateListData,
    out_lsd: LstStateData,
    inp_lsd: LstStateData,
    rebalance_auth: [u8; 32],
    out_balance: u64,
    inp_balance: u64,
    modify_pool: Option<fn(&mut PoolState)>,
    modify_lsds: Option<fn(&mut LstStateData, &mut LstStateData)>,
    create_ix_args: impl FnOnce(u32, u32) -> StartRebalanceIxArgs,
    expected_err: impl Into<ProgramError>,
) -> TestCaseResult {
    let mut out_lsd = out_lsd;
    out_lsd.lst_state.sol_value_calculator = *SvcAgTy::Wsol(()).svc_program_id();
    let mut inp_lsd = inp_lsd;
    inp_lsd.lst_state.sol_value_calculator = *SvcAgTy::Wsol(()).svc_program_id();

    if let Some(modifier) = modify_lsds {
        modifier(&mut out_lsd, &mut inp_lsd);
    }

    let withdraw_to = Pubkey::new_unique().to_bytes();
    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);
    let builder = wsol_builder(
        rebalance_auth,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    let mut pool = pool;
    if let Some(modifier) = modify_pool {
        modifier(&mut pool);
    }

    let ix_args = create_ix_args(out_idx, inp_idx);

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        ix_args,
        out_balance,
        inp_balance,
        withdraw_to,
        true,
        None,
        [],
        Some(expected_err),
    )
}

#[test]
fn start_rebalance_instructions_sysvar_variants() {
    let JupSolFixtureData {
        rebalance_auth,
        out_mint,
        inp_mint,
        inp_idx,
        ..
    } = load_jupsol_fixture();

    let withdraw_to = Pubkey::new_unique().to_bytes();
    let builder = jupsol_wsol_builder(rebalance_auth, out_mint, inp_mint, withdraw_to);

    let mut accounts = start_rebalance_fixtures_accounts(&builder);
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(rebalance_auth),
            Account {
                lamports: u64::MAX,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Default::default()
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
            Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
            keyed_account_for_system_program().1,
        ),
    );

    let mut instruction_sets = Vec::new();

    let base_instructions = rebalance_ixs(
        &builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        inp_idx as u32,
        250_000,
        0,
        u64::MAX,
    );
    instruction_sets.push(base_instructions.clone());

    // EndRebalance in the middle of additional ixs
    let mut with_middle_end = base_instructions.clone();
    with_middle_end.insert(1, base_instructions[1].clone());
    instruction_sets.push(with_middle_end);

    // Multiple EndRebalance ixs
    let mut with_multiple_end = base_instructions.clone();
    with_multiple_end.push(base_instructions[1].clone());
    instruction_sets.push(with_multiple_end);

    for instructions in instruction_sets {
        let mut scenario_accounts = accounts.clone();
        let rebalance_record_pda = Pubkey::new_from_array(REBALANCE_RECORD_ID);
        upsert_account(
            &mut scenario_accounts,
            (rebalance_record_pda, mock_empty_rebalance_record_account()),
        );
        upsert_account(
            &mut scenario_accounts,
            instructions_sysvar(&instructions, 0),
        );

        let InstructionResult {
            program_result,
            resulting_accounts,
            ..
        } = SVM.with(|svm| svm.process_instruction(&instructions[0], &scenario_accounts));

        assert_eq!(program_result, ProgramResult::Success);
        assert_start_success(
            &scenario_accounts,
            &resulting_accounts,
            &out_mint,
            &inp_mint,
        );
    }
}

#[test]
fn start_rebalance_missing_end_rebalance_fails() {
    let (pool, mut lsl, out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let builder = wsol_builder(
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        default_ix_args(out_idx, inp_idx, 100_000),
        200_000,
        150_000,
        withdraw_to,
        false,
        None,
        [],
        Some(Inf1CtlCustomProgErr(Inf1CtlErr::NoSucceedingEndRebalance)),
    )
    .unwrap();
}

#[test]
fn start_rebalance_wrong_end_mint_fails() {
    let JupSolFixtureData {
        rebalance_auth,
        out_mint,
        inp_mint,
        inp_idx,
        ..
    } = load_jupsol_fixture();

    let withdraw_to = Pubkey::new_unique().to_bytes();
    let builder = jupsol_wsol_builder(rebalance_auth, out_mint, inp_mint, withdraw_to);

    let mut accounts = start_rebalance_fixtures_accounts(&builder);
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(rebalance_auth),
            Account {
                lamports: u64::MAX,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Default::default()
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
            Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
            keyed_account_for_system_program().1,
        ),
    );

    let mut instructions = rebalance_ixs(
        &builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        inp_idx as u32,
        100_000,
        0,
        u64::MAX,
    );

    if let Some(end_ix) = instructions.get_mut(1) {
        if end_ix.accounts.len() > END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT {
            end_ix.accounts[END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT].pubkey =
                Pubkey::new_unique();
        }
    }

    upsert_account(&mut accounts, instructions_sysvar(&instructions, 0));

    let rebalance_record_pda = Pubkey::new_from_array(REBALANCE_RECORD_ID);
    upsert_account(
        &mut accounts,
        (rebalance_record_pda, mock_empty_rebalance_record_account()),
    );

    let InstructionResult { program_result, .. } =
        SVM.with(|svm| svm.process_instruction(&instructions[0], &accounts));

    inf1_test_utils::assert_jiminy_prog_err(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::NoSucceedingEndRebalance),
    );
}

fn access_control_inputs() -> impl Strategy<
    Value = (
        PoolState,
        LstStateData,
        LstStateData,
        [u8; 32],
        u64,
        u64,
        u64,
    ),
> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal(),
        ..Default::default()
    })
    .prop_flat_map(|pool| {
        let auth = pool.rebalance_authority;
        (
            Just(pool),
            any_lst_state(
                AnyLstStateArgs {
                    sol_value: Some((0..=u64::MAX / 2).boxed()),
                    is_input_disabled: Some(Just(false).boxed()),
                    ..Default::default()
                },
                None,
            ),
            any_lst_state(
                AnyLstStateArgs {
                    sol_value: Some((0..=u64::MAX / 2).boxed()),
                    is_input_disabled: Some(Just(false).boxed()),
                    ..Default::default()
                },
                None,
            ),
            any_normal_pk().prop_filter("cannot reuse rebalance authority", move |pk| *pk != auth),
            1u64..=1_000_000_000,
            1u64..=1_000_000_000,
            1u64..=1_000_000_000,
        )
    })
    .prop_map(
        |(pool, out_lsd, inp_lsd, non_auth, amount, out_balance, inp_balance)| {
            (
                pool,
                out_lsd,
                inp_lsd,
                non_auth,
                amount,
                out_balance,
                inp_balance,
            )
        },
    )
}

fn validation_inputs(
) -> impl Strategy<Value = (PoolState, LstStateData, LstStateData, u64, u64, u64)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal(),
        ..Default::default()
    })
    .prop_flat_map(|pool| {
        (
            Just(pool),
            any_lst_state(
                AnyLstStateArgs {
                    sol_value: Some((0..=u64::MAX / 2).boxed()),
                    is_input_disabled: Some(Just(false).boxed()),
                    ..Default::default()
                },
                None,
            ),
            any_lst_state(
                AnyLstStateArgs {
                    sol_value: Some((0..=u64::MAX / 2).boxed()),
                    is_input_disabled: Some(Just(false).boxed()),
                    ..Default::default()
                },
                None,
            ),
            1u64..=1_000_000_000,
            1u64..=1_000_000_000,
            1u64..=1_000_000_000,
        )
    })
}

proptest! {
    #[test]
    fn start_rebalance_unauthorized_any(
        (pool, out_lsd, inp_lsd, non_auth, amount, out_balance, inp_balance) in access_control_inputs(),
        lsl in any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
    ) {
        run_wsol_proptest_case(
            pool,
            lsl,
            out_lsd,
            inp_lsd,
            non_auth,
            out_balance,
            inp_balance,
            None,
            None,
            |out_idx, inp_idx| default_ix_args(out_idx, inp_idx, amount),
            INVALID_ARGUMENT,
        ).unwrap();
    }
}

proptest! {
    #[test]
    fn start_rebalance_pool_rebalancing_any(
        (pool, out_lsd, inp_lsd, _non_auth, amount, out_balance, inp_balance) in access_control_inputs(),
        lsl in any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
    ) {
        let rebalance_auth = pool.rebalance_authority;
        run_wsol_proptest_case(
            pool,
            lsl,
            out_lsd,
            inp_lsd,
            rebalance_auth,
            out_balance,
            inp_balance,
            Some(|pool| { U8BoolMut(&mut pool.is_rebalancing).set_true(); }),
            None,
            |out_idx, inp_idx| default_ix_args(out_idx, inp_idx, amount),
            Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing),
          ).unwrap();
    }
}

proptest! {
    #[test]
    fn start_rebalance_pool_disabled_any(
        (pool, out_lsd, inp_lsd, _non_auth, amount, out_balance, inp_balance) in access_control_inputs(),
        lsl in any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
    ) {
        let rebalance_auth = pool.rebalance_authority;
        run_wsol_proptest_case(
            pool,
            lsl,
            out_lsd,
            inp_lsd,
            rebalance_auth,
            out_balance,
            inp_balance,
            Some(|pool| { U8BoolMut(&mut pool.is_disabled).set_true(); }),
            None,
            |out_idx, inp_idx| default_ix_args(out_idx, inp_idx, amount),
            Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled),
        ).unwrap();
    }
}

proptest! {
    #[test]
    fn start_rebalance_dest_disabled_any(
        (pool, out_lsd, inp_lsd, amount, out_balance, inp_balance) in validation_inputs(),
        lsl in any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
    ) {
        let rebalance_auth = pool.rebalance_authority;
        run_wsol_proptest_case(
            pool,
            lsl,
            out_lsd,
            inp_lsd,
            rebalance_auth,
            out_balance,
            inp_balance,
            None,
            Some(|_, inp_lsd| { U8BoolMut(&mut inp_lsd.lst_state.is_input_disabled).set_true(); }),
            |out_idx, inp_idx| default_ix_args(out_idx, inp_idx, amount),
            Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled),
        ).unwrap();
    }
}

proptest! {
    #[test]
    fn start_rebalance_invalid_out_index_any(
        (pool, out_lsd, inp_lsd, amount, out_balance, inp_balance) in validation_inputs(),
        lsl in any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
    ) {
        let rebalance_auth = pool.rebalance_authority;
        run_wsol_proptest_case(
            pool,
            lsl,
            out_lsd,
            inp_lsd,
            rebalance_auth,
            out_balance,
            inp_balance,
            None,
            None,
            |_, inp_idx| default_ix_args((MAX_LST_STATES as u32) + 5, inp_idx, amount),
            Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex),
        ).unwrap();
    }
}

proptest! {
    #[test]
    fn start_rebalance_invalid_inp_index_any(
        (pool, out_lsd, inp_lsd, amount, out_balance, inp_balance) in validation_inputs(),
        lsl in any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
    ) {
        let rebalance_auth = pool.rebalance_authority;
        run_wsol_proptest_case(
            pool,
            lsl,
            out_lsd,
            inp_lsd,
            rebalance_auth,
            out_balance,
            inp_balance,
            None,
            None,
            |out_idx, _inp_idx| default_ix_args(out_idx, (MAX_LST_STATES as u32) + 5, amount),
            Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex),
        ).unwrap();
    }
}

#[test]
fn start_rebalance_min_out_slippage_fails() {
    let (pool, mut lsl, out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let builder = wsol_builder(
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        StartRebalanceIxArgs {
            min_starting_out_lst: 60_000,
            ..default_ix_args(out_idx, inp_idx, 100_000)
        },
        50_000,
        50_000,
        withdraw_to,
        true,
        None,
        [],
        Some(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded)),
    )
    .unwrap();
}

#[test]
fn start_rebalance_max_in_slippage_fails() {
    let (pool, mut lsl, out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let builder = wsol_builder(
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        StartRebalanceIxArgs {
            max_starting_inp_lst: 40_000,
            ..default_ix_args(out_idx, inp_idx, 100_000)
        },
        50_000,
        50_000,
        withdraw_to,
        true,
        None,
        [],
        Some(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded)),
    )
    .unwrap();
}

#[test]
fn start_rebalance_calc_program_mismatch_fails() {
    let (pool, mut lsl, out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();
    let wrong_prog = Pubkey::new_unique().to_bytes();

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let ix_prefix = start_rebalance_ix_pre_keys_owned(
        pool.rebalance_authority,
        &TOKENKEG_PROGRAM,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    let builder = StartRebalanceKeysBuilder {
        ix_prefix,
        out_calc_prog: wrong_prog,
        out_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    };

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        default_ix_args(out_idx, inp_idx, 100_000),
        200_000,
        150_000,
        withdraw_to,
        true,
        None,
        [],
        Some(INVALID_ARGUMENT),
    )
    .unwrap();
}

#[test]
fn start_rebalance_invalid_reserves_fails() {
    let (pool, mut lsl, out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();

    let wrong_reserves_pk = Pubkey::new_unique();
    let wrong_reserves_acc = mock_token_acc(raw_token_acc(
        out_lsd.lst_state.mint,
        POOL_STATE_ID,
        200_000,
    ));

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let builder = wsol_builder(
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        default_ix_args(out_idx, inp_idx, 100_000),
        200_000,
        150_000,
        withdraw_to,
        true,
        None,
        [(wrong_reserves_pk, wrong_reserves_acc)],
        Some(INVALID_ARGUMENT),
    )
    .unwrap();
}

#[test]
fn start_rebalance_zero_out_calc_accounts_fails() {
    let (pool, mut lsl, out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let builder = wsol_builder(
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    let ix_args = default_ix_args(out_idx, inp_idx, 100_000);

    let override_args = StartRebalanceIxArgs {
        out_lst_value_calc_accs: 0,
        ..ix_args
    };

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        ix_args,
        200_000,
        150_000,
        withdraw_to,
        true,
        Some(override_args),
        [],
        Some(NOT_ENOUGH_ACCOUNT_KEYS),
    )
    .unwrap();
}

#[test]
fn start_rebalance_missing_suffix_account_fails() {
    let (pool, mut lsl, mut out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();

    // Simulate a scenario where the instruction expects more accounts than are present
    let calc_suf = jupsol_fixtures_svc_suf();
    let actual_count = (calc_suf.suf_len() + 1) as u8; // +1 for the program itself
    let inflated_count = actual_count + 5;

    out_lsd.lst_state.sol_value_calculator = *SvcAgTy::SanctumSplMulti(()).svc_program_id();

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let ix_prefix = start_rebalance_ix_pre_keys_owned(
        pool.rebalance_authority,
        &TOKENKEG_PROGRAM,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    let builder = StartRebalanceKeysBuilder {
        ix_prefix,
        out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        out_calc: calc_suf,
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    };

    let ix_args = StartRebalanceIxArgs {
        out_lst_value_calc_accs: actual_count,
        ..default_ix_args(out_idx, inp_idx, 100_000)
    };

    let override_args = StartRebalanceIxArgs {
        out_lst_value_calc_accs: inflated_count,
        ..ix_args
    };

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        ix_args,
        200_000,
        150_000,
        withdraw_to,
        true,
        Some(override_args),
        [],
        Some(NOT_ENOUGH_ACCOUNT_KEYS),
    )
    .unwrap();
}

#[test]
fn start_rebalance_invalid_inp_reserves_fails() {
    let (pool, mut lsl, out_lsd, inp_lsd) = fixture_lst_state_data();
    let withdraw_to = Pubkey::new_unique().to_bytes();

    let wrong_inp_reserves_pk = Pubkey::new_unique();
    let wrong_inp_reserves_acc = mock_token_acc(raw_token_acc(
        inp_lsd.lst_state.mint,
        POOL_STATE_ID,
        150_000,
    ));

    let (out_idx, inp_idx) = compute_lst_indices(&mut lsl, out_lsd, inp_lsd);

    let builder = wsol_builder(
        pool.rebalance_authority,
        out_lsd.lst_state.mint,
        inp_lsd.lst_state.mint,
        withdraw_to,
    );

    run_start_case(
        pool,
        lsl,
        out_lsd,
        inp_lsd,
        builder,
        default_ix_args(out_idx, inp_idx, 100_000),
        200_000,
        150_000,
        withdraw_to,
        true,
        None,
        [(wrong_inp_reserves_pk, wrong_inp_reserves_acc)],
        Some(INVALID_ARGUMENT),
    )
    .unwrap();
}

#[test]
fn start_rebalance_jupsol_fixture_snapshot() {
    silence_mollusk_logs();

    let JupSolFixtureData {
        pool,
        lst_state_bytes,
        lst_states,
        rebalance_auth,
        out_mint,
        inp_mint,
        inp_idx,
    } = load_jupsol_fixture();

    let withdraw_to = Pubkey::new_unique().to_bytes();
    let builder = jupsol_wsol_builder(rebalance_auth, out_mint, inp_mint, withdraw_to);

    let instructions = rebalance_ixs(
        &builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        inp_idx as u32,
        250_000,
        0,
        u64::MAX,
    );

    let mut accounts = start_rebalance_fixtures_accounts(&builder);
    let out_mint_bytes = lst_states[JUPSOL_FIXTURE_LST_IDX].mint;
    let inp_mint_bytes = lst_states[inp_idx].mint;

    add_common_accounts(
        &mut accounts,
        &pool,
        &lst_state_bytes,
        None,
        rebalance_auth,
        out_mint_bytes,
        inp_mint_bytes,
        withdraw_to,
        500_000,
        300_000,
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(rebalance_auth),
            solana_account::Account {
                lamports: u64::MAX,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Default::default()
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
            Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
            keyed_account_for_system_program().1,
        ),
    );

    {
        let out_prog = Pubkey::new_from_array(builder.out_calc_prog);
        let calc_acc = mock_prog_acc(Pubkey::new_unique());
        upsert_account(&mut accounts, (out_prog, calc_acc));
    }

    {
        let inp_prog = Pubkey::new_from_array(builder.inp_calc_prog);
        let calc_acc = mock_prog_acc(Pubkey::new_unique());
        upsert_account(&mut accounts, (inp_prog, calc_acc));
    }

    upsert_account(&mut accounts, instructions_sysvar(&instructions, 0));

    let rebalance_record_pda = Pubkey::new_from_array(REBALANCE_RECORD_ID);
    upsert_account(
        &mut accounts,
        (rebalance_record_pda, mock_empty_rebalance_record_account()),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&instructions[0], &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    // Verify rebalance record was created with correct contents
    let rebalance_record_acc = resulting_accounts
        .iter()
        .find(|(pk, _)| *pk == rebalance_record_pda)
        .map(|(_, acc)| acc)
        .expect("rebalance record account");

    assert_eq!(
        rebalance_record_acc.data.len(),
        size_of::<RebalanceRecord>(),
        "rebalance record should have correct size"
    );

    let record_packed = RebalanceRecordPacked::of_acc_data(&rebalance_record_acc.data)
        .expect("rebalance record should be valid");
    let record: RebalanceRecord = (*record_packed).into();

    let inp_lst_index_str = format!("{}", record.inp_lst_index);
    expect!["0"].assert_eq(&inp_lst_index_str);

    assert!(
        record.old_total_sol_value > 0,
        "starting pool value should be positive"
    );

    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_bef = accounts
        .iter()
        .find(|(pk, _)| *pk == pool_pk)
        .map(|(_, acc)| acc)
        .expect("pool account before");
    let pool_aft = resulting_accounts
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

    assert_eq!(
        pool_state_bef.is_rebalancing, 0,
        "pool should start not rebalancing"
    );
    assert_eq!(
        pool_state_aft.is_rebalancing, 1,
        "pool should end rebalancing"
    );

    // Verify the rebalance record stores a reasonable pool value
    assert!(
        record.old_total_sol_value > 0,
        "rebalance record should store positive pool value"
    );
    assert!(
        (record.old_total_sol_value as i128 - pool_state_aft.total_sol_value as i128).abs()
            < 1_000_000,
        "rebalance record value should be close to final pool value"
    );
}

pub fn fixture_lst_state_data() -> (PoolState, LstStateListData, LstStateData, LstStateData) {
    let (pool, mut lst_state_bytes) = fixture_pool_and_lsl();

    let packed_list = LstStatePackedList::of_acc_data(&lst_state_bytes).expect("lst packed");
    let packed_states = &packed_list.0;

    let out_state = packed_states[JUPSOL_FIXTURE_LST_IDX].into_lst_state();
    let inp_state = packed_states
        .iter()
        .find(|s| s.into_lst_state().mint == WSOL_MINT.to_bytes())
        .expect("wsol fixture available")
        .into_lst_state();

    let mut out_state = out_state;
    out_state.sol_value_calculator = *SvcAgTy::Wsol(()).svc_program_id();
    let mut inp_state = inp_state;
    inp_state.sol_value_calculator = *SvcAgTy::Wsol(()).svc_program_id();

    let out_protocol = create_protocol_fee_accumulator_ata(
        &TOKENKEG_PROGRAM,
        &out_state.mint,
        out_state.protocol_fee_accumulator_bump,
    )
    .to_bytes();
    let out_reserves = create_pool_reserves_ata(
        &TOKENKEG_PROGRAM,
        &out_state.mint,
        out_state.pool_reserves_bump,
    )
    .to_bytes();

    let inp_protocol = create_protocol_fee_accumulator_ata(
        &TOKENKEG_PROGRAM,
        &inp_state.mint,
        inp_state.protocol_fee_accumulator_bump,
    )
    .to_bytes();
    let inp_reserves = create_pool_reserves_ata(
        &TOKENKEG_PROGRAM,
        &inp_state.mint,
        inp_state.pool_reserves_bump,
    )
    .to_bytes();

    let out_lsd = LstStateData {
        lst_state: out_state,
        protocol_fee_accumulator: out_protocol,
        pool_reserves: out_reserves,
    };

    let inp_lsd = LstStateData {
        lst_state: inp_state,
        protocol_fee_accumulator: inp_protocol,
        pool_reserves: inp_reserves,
    };

    {
        let list_mut = LstStatePackedListMut::of_acc_data(&mut lst_state_bytes).unwrap();
        if let Some(packed) = list_mut.0.get_mut(JUPSOL_FIXTURE_LST_IDX) {
            unsafe {
                packed.as_lst_state_mut().sol_value_calculator =
                    out_lsd.lst_state.sol_value_calculator;
            }
        }
        if let Some(packed) = list_mut
            .0
            .iter_mut()
            .find(|packed| packed.into_lst_state().mint == inp_lsd.lst_state.mint)
        {
            unsafe {
                packed.as_lst_state_mut().sol_value_calculator =
                    inp_lsd.lst_state.sol_value_calculator;
            }
        }
    }

    let mut lsl_data = LstStateListData {
        lst_state_list: lst_state_bytes,
        protocol_fee_accumulators: HashMap::new(),
        all_pool_reserves: HashMap::new(),
    };
    lsl_data
        .protocol_fee_accumulators
        .insert(out_lsd.lst_state.mint, out_lsd.protocol_fee_accumulator);
    lsl_data
        .protocol_fee_accumulators
        .insert(inp_lsd.lst_state.mint, inp_lsd.protocol_fee_accumulator);
    lsl_data
        .all_pool_reserves
        .insert(out_lsd.lst_state.mint, out_lsd.pool_reserves);
    lsl_data
        .all_pool_reserves
        .insert(inp_lsd.lst_state.mint, inp_lsd.pool_reserves);

    (pool, lsl_data, out_lsd, inp_lsd)
}

#[allow(clippy::too_many_arguments)]
pub fn add_common_accounts(
    accounts: &mut Vec<PkAccountTup>,
    pool: &PoolState,
    lst_state_list: &[u8],
    pool_reserves_map: Option<&HashMap<[u8; 32], [u8; 32]>>,
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
    out_balance: u64,
    inp_balance: u64,
) {
    upsert_account(
        accounts,
        (
            LST_STATE_LIST_ID.into(),
            lst_state_list_account(lst_state_list.to_vec()),
        ),
    );
    upsert_account(accounts, (POOL_STATE_ID.into(), pool_state_account(*pool)));
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(rebalance_auth),
            Account {
                lamports: u64::MAX,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Default::default()
            },
        ),
    );
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(out_mint),
            mock_mint(raw_mint(None, None, 0, 9)),
        ),
    );
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(inp_mint),
            mock_mint(raw_mint(None, None, 0, 9)),
        ),
    );
    upsert_account(
        accounts,
        (
            pool_reserves_map
                .and_then(|m| m.get(&out_mint).copied())
                .map(Pubkey::new_from_array)
                .unwrap_or_else(|| {
                    inf1_test_utils::find_pool_reserves_ata(&TOKENKEG_PROGRAM, &out_mint).0
                }),
            mock_token_acc(raw_token_acc(out_mint, POOL_STATE_ID, out_balance)),
        ),
    );
    upsert_account(
        accounts,
        (
            pool_reserves_map
                .and_then(|m| m.get(&inp_mint).copied())
                .map(Pubkey::new_from_array)
                .unwrap_or_else(|| {
                    inf1_test_utils::find_pool_reserves_ata(&TOKENKEG_PROGRAM, &inp_mint).0
                }),
            mock_token_acc(raw_token_acc(inp_mint, POOL_STATE_ID, inp_balance)),
        ),
    );
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(withdraw_to),
            mock_token_acc(raw_token_acc(out_mint, withdraw_to, 0)),
        ),
    );
}

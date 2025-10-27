use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::{PoolState, PoolStatePacked},
    },
    err::Inf1CtlErr,
    instructions::set_sol_value_calculator::{
        NewSetSolValueCalculatorIxPreAccsBuilder, SetSolValueCalculatorIxData,
        SetSolValueCalculatorIxPreKeysOwned,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};

use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_spl_core::instructions::sol_val_calc::SanctumSplMultiCalcAccs,
    inf1_svc_spl_core::keys::sanctum_spl_multi,
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAgTy,
};

use inf1_core::instructions::set_sol_value_calculator::{
    set_sol_value_calculator_ix_is_signer, set_sol_value_calculator_ix_is_writer,
    set_sol_value_calculator_ix_keys_owned, SetSolValueCalculatorIxAccs,
};

use inf1_test_utils::{
    acc_bef_aft, any_lst_state, any_lst_state_list, any_normal_pk, any_pool_state,
    any_spl_stake_pool, any_wsol_lst_state, assert_jiminy_prog_err, find_pool_reserves,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, lst_state_list_account, mock_mint,
    mock_spl_stake_pool, mock_token_acc, pool_state_account, raw_mint, raw_token_acc,
    silence_mollusk_logs, upsert_account, GenLstStateArgs, GenPoolStateArgs, GenStakePoolArgs,
    LstStateData, LstStateListData, LstStatePks, NewLstStatePksBuilder, NewPoolStateBoolsBuilder,
    NewSplStakePoolU64sBuilder, PkAccountTup, PoolStateBools, SplStakePoolU64s, ALL_FIXTURES,
    JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT,
};

use jiminy_cpi::program_error::INVALID_ARGUMENT;
use proptest::{prelude::*, test_runner::TestCaseResult};

use mollusk_svm::result::{InstructionResult, ProgramResult};

use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{
    jupsol_fixtures_svc_suf, max_sol_val_no_overflow, MAX_LAMPORTS_OVER_SUPPLY, MAX_LST_STATES, SVM,
};

type SetSolValueCalculatorKeysBuilder =
    SetSolValueCalculatorIxAccs<[u8; 32], SetSolValueCalculatorIxPreKeysOwned, SvcCalcAccsAg>;

fn set_sol_value_calculator_ix_pre_keys_owned(
    admin: [u8; 32],
    token_program: &[u8; 32],
    mint: [u8; 32],
) -> SetSolValueCalculatorIxPreKeysOwned {
    NewSetSolValueCalculatorIxPreAccsBuilder::start()
        .with_admin(admin)
        .with_lst_mint(mint)
        .with_pool_state(POOL_STATE_ID)
        .with_pool_reserves(find_pool_reserves(token_program, &mint).0.to_bytes())
        .with_lst_state_list(LST_STATE_LIST_ID)
        .build()
}

fn set_sol_value_calculator_ix(
    builder: &SetSolValueCalculatorKeysBuilder,
    lst_idx: u32,
) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        set_sol_value_calculator_ix_keys_owned(builder).seq(),
        set_sol_value_calculator_ix_is_signer(builder).seq(),
        set_sol_value_calculator_ix_is_writer(builder).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetSolValueCalculatorIxData::new(lst_idx).as_buf().into(),
    }
}

fn set_sol_value_calculator_fixtures_accounts_opt(
    builder: &SetSolValueCalculatorKeysBuilder,
) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(
        set_sol_value_calculator_ix_keys_owned(builder)
            .seq()
            .copied(),
    )
    .collect()
}

pub fn assert_correct_set(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    mint: &[u8; 32],
    expected_new_calc: &[u8; 32],
) {
    let [pools, lst_state_lists] = [POOL_STATE_ID, LST_STATE_LIST_ID]
        .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), bef, aft));
    let [pool_bef, pool_aft] = pools.each_ref().map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state()
    });
    let [lst_state_list_bef, lst_state_list_aft] = lst_state_lists
        .each_ref()
        .map(|a| LstStatePackedList::of_acc_data(&a.data).unwrap());
    let lst_state_i = lst_state_list_bef
        .0
        .iter()
        .position(|s| s.into_lst_state().mint == *mint)
        .unwrap();
    let [lst_state_bef, lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[lst_state_i].into_lst_state());

    assert_eq!(lst_state_bef.mint, *mint);
    assert_eq!(lst_state_bef.mint, lst_state_aft.mint);

    assert_ne!(
        lst_state_bef.sol_value_calculator,
        lst_state_aft.sol_value_calculator
    );
    assert_eq!(&lst_state_aft.sol_value_calculator, expected_new_calc);

    let delta = i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value);
    assert_eq!(
        delta,
        i128::from(lst_state_aft.sol_value) - i128::from(lst_state_bef.sol_value)
    );
}

#[test]
fn set_sol_value_calculator_jupsol_fixture() {
    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_acc = ALL_FIXTURES
        .get(&pool_pk)
        .expect("missing pool state fixture");

    let pool = PoolStatePacked::of_acc_data(&pool_acc.data)
        .unwrap()
        .into_pool_state();
    let admin = pool.admin;

    let ix_prefix = set_sol_value_calculator_ix_pre_keys_owned(
        admin,
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
    );
    let builder = SetSolValueCalculatorKeysBuilder {
        ix_prefix,
        calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        calc: jupsol_fixtures_svc_suf(),
    };
    let ix = set_sol_value_calculator_ix(&builder, JUPSOL_FIXTURE_LST_IDX as u32);
    let mut accounts = set_sol_value_calculator_fixtures_accounts_opt(&builder);

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(admin),
            Account {
                lamports: u64::MAX,
                ..Default::default()
            },
        ),
    );

    let lsl_pk = Pubkey::new_from_array(LST_STATE_LIST_ID);
    let lsl_acc = ALL_FIXTURES.get(&lsl_pk).unwrap().clone();
    let mut lsl_data = lsl_acc.data.to_vec();

    // Set initial calculator to a random pubkey
    let lsl_mut = LstStatePackedListMut::of_acc_data(&mut lsl_data).unwrap();
    let lst_mut = unsafe {
        lsl_mut
            .0
            .get_mut(JUPSOL_FIXTURE_LST_IDX)
            .unwrap()
            .as_lst_state_mut()
    };
    lst_mut.sol_value_calculator = Pubkey::new_unique().to_bytes();

    upsert_account(&mut accounts, (lsl_pk, lst_state_list_account(lsl_data)));

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_set(
        &accounts,
        &resulting_accounts,
        JUPSOL_MINT.as_array(),
        jupsol_fixtures_svc_suf().svc_program_id(),
    );
}

enum TestErrorType {
    Unauthorized,
    PoolRebalancing,
    PoolDisabled,
}

#[allow(clippy::too_many_arguments)]
fn set_sol_value_calculator_proptest(
    pool: PoolState,
    mut lsl: LstStateListData,
    lsd: LstStateData,
    admin: [u8; 32],
    calc_prog: [u8; 32],
    calc: SvcCalcAccsAg,
    initial_calc_prog: [u8; 32],
    new_balance: u64,
    additional_accounts: impl IntoIterator<Item = PkAccountTup>,
    error_type: Option<TestErrorType>,
) -> TestCaseResult {
    silence_mollusk_logs();
    let lst_idx = lsl.upsert(lsd);
    let LstStateListData {
        lst_state_list,
        all_pool_reserves,
        ..
    } = lsl;
    let mint = lsd.lst_state.mint;

    // Set initial calculator to a random pubkey
    let mut lsl_data = lst_state_list.clone();
    let lsl_mut = LstStatePackedListMut::of_acc_data(&mut lsl_data).unwrap();
    let lst_mut = unsafe { lsl_mut.0.get_mut(lst_idx).unwrap().as_lst_state_mut() };
    lst_mut.sol_value_calculator = initial_calc_prog;

    let ix_prefix = set_sol_value_calculator_ix_pre_keys_owned(admin, &TOKENKEG_PROGRAM, mint);
    let builder = SetSolValueCalculatorKeysBuilder {
        ix_prefix,
        calc_prog,
        calc,
    };

    let ix = set_sol_value_calculator_ix(&builder, lst_idx as u32);
    let mut accounts = set_sol_value_calculator_fixtures_accounts_opt(&builder);

    // Common upserts
    upsert_account(
        &mut accounts,
        (LST_STATE_LIST_ID.into(), lst_state_list_account(lsl_data)),
    );
    upsert_account(
        &mut accounts,
        (POOL_STATE_ID.into(), pool_state_account(pool)),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(admin),
            Account {
                lamports: u64::MAX,
                ..Default::default()
            },
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(*all_pool_reserves.get(&mint).unwrap()),
            mock_token_acc(raw_token_acc(mint, POOL_STATE_ID, new_balance)),
        ),
    );

    // Additional test-specific upserts
    additional_accounts
        .into_iter()
        .for_each(|account| upsert_account(&mut accounts, account));

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    if let Some(error_type) = error_type {
        match error_type {
            TestErrorType::Unauthorized => {
                assert_jiminy_prog_err(&program_result, INVALID_ARGUMENT);
            }
            TestErrorType::PoolRebalancing => {
                assert_jiminy_prog_err(
                    &program_result,
                    Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing),
                );
            }
            TestErrorType::PoolDisabled => {
                assert_jiminy_prog_err(
                    &program_result,
                    Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled),
                );
            }
        }
    } else {
        prop_assert_eq!(program_result, ProgramResult::Success);
        assert_correct_set(&accounts, &resulting_accounts, &mint, &calc_prog);
    }

    Ok(())
}

proptest! {
    #[test]
    fn set_sol_value_calculator_unauthorized_any(
        (pool, lsd, stake_pool_addr, stake_pool, non_admin, initial_svc_addr, new_balance) in
            (any_pool_state(GenPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            }),
            any_normal_pk(),
            any::<u64>(),
        ).prop_flat_map(
                |(pool, mint_addr, spl_lamports)| (
                    Just(pool),
                    any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != mint_addr),
                    any_spl_stake_pool(GenStakePoolArgs {
                        pool_mint: Some(Just(mint_addr).boxed()),
                        u64s: SplStakePoolU64s(NewSplStakePoolU64sBuilder::start()
                            .with_last_update_epoch(Just(0).boxed()) // mollusk clock defaults to epoch 0
                            .with_total_lamports(Just(spl_lamports).boxed())
                            .with_pool_token_supply((spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed())
                            .build().0.map(Some)),
                        ..Default::default()
                    }),
                    any_lst_state(GenLstStateArgs {
                        sol_value: Some((0..=pool.total_sol_value).boxed()),
                        pks: LstStatePks(NewLstStatePksBuilder::start()
                        .with_mint(mint_addr)
                        .with_sol_value_calculator(sanctum_spl_multi::ID)
                        .build().0.map(|x| Some(Just(x).boxed()))),
                        ..Default::default()
                    }),
                    any_normal_pk().prop_filter("cannot be eq admin", move |x| *x != pool.admin),
                )
            ).prop_flat_map(
                |(pool, stake_pool_addr, stake_pool, lsd, non_admin)| (
                    Just(pool),
                    Just(lsd),
                    Just(stake_pool_addr),
                    Just(stake_pool),
                    Just(non_admin),
                    any_normal_pk(),
                    0..=max_sol_val_no_overflow(pool.total_sol_value, lsd.lst_state.sol_value) / MAX_LAMPORTS_OVER_SUPPLY,
                )
            ),
        lsl in any_lst_state_list(Default::default(), 0..=MAX_LST_STATES),
    ) {
        set_sol_value_calculator_proptest(pool, lsl, lsd, non_admin, *SvcAgTy::SanctumSplMulti(()).svc_program_id(), SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }), initial_svc_addr, new_balance, [
            (lsd.lst_state.mint.into(), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            (Pubkey::new_from_array(stake_pool_addr), mock_spl_stake_pool(stake_pool, sanctum_spl_multi::POOL_PROG_ID.into())),
        ], Some(TestErrorType::Unauthorized)).unwrap();
    }
}

proptest! {
    #[test]
    fn set_sol_value_calculator_rebalancing_any(
        (pool, lsd, stake_pool_addr, stake_pool, initial_svc_addr, new_balance) in
        (any_pool_state(GenPoolStateArgs {
            bools: PoolStateBools(NewPoolStateBoolsBuilder::start()
            .with_is_disabled(false)
            .with_is_rebalancing(true)
            .build().0.map(|x| Some(Just(x).boxed()))),
            ..Default::default()
        }),
            any_normal_pk(),
            any::<u64>(),
        ).prop_flat_map(
                |(pool, mint_addr, spl_lamports)| (
                    Just(pool),
                    any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != mint_addr),
                    any_spl_stake_pool(GenStakePoolArgs {
                        pool_mint: Some(Just(mint_addr).boxed()),
                        u64s: SplStakePoolU64s(NewSplStakePoolU64sBuilder::start()
                            .with_last_update_epoch(Just(0).boxed()) // mollusk clock defaults to epoch 0
                            .with_total_lamports(Just(spl_lamports).boxed())
                            .with_pool_token_supply((spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed())
                            .build().0.map(Some)),
                        ..Default::default()
                    }),
                    any_lst_state(GenLstStateArgs {
                        sol_value: Some((0..=pool.total_sol_value).boxed()),
                        pks: LstStatePks(NewLstStatePksBuilder::start()
                        .with_mint(mint_addr)
                        .with_sol_value_calculator(sanctum_spl_multi::ID)
                        .build().0.map(|x| Some(Just(x).boxed()))),
                        ..Default::default()
                    }),
                )
            ).prop_flat_map(
                |(pool, stake_pool_addr, stake_pool, lsd)| (
                    Just(pool),
                    Just(lsd),
                    Just(stake_pool_addr),
                    Just(stake_pool),
                    any_normal_pk(),
                    0..=max_sol_val_no_overflow(pool.total_sol_value, lsd.lst_state.sol_value) / MAX_LAMPORTS_OVER_SUPPLY,
                )
            ),
        lsl in any_lst_state_list(Default::default(), 0..=MAX_LST_STATES),
    ) {
        set_sol_value_calculator_proptest(pool, lsl, lsd, pool.admin, *SvcAgTy::SanctumSplMulti(()).svc_program_id(), SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }), initial_svc_addr, new_balance, [
                (
                    lsd.lst_state.mint.into(),
                    mock_mint(raw_mint(None, None, u64::MAX, 9)),
                ),
                (lsd.lst_state.mint.into(), mock_mint(raw_mint(None, None, u64::MAX, 9))),
                (Pubkey::new_from_array(stake_pool_addr), mock_spl_stake_pool(stake_pool, sanctum_spl_multi::POOL_PROG_ID.into())),
            ], Some(TestErrorType::PoolRebalancing)).unwrap();
    }
}

proptest! {
    #[test]
    fn set_sol_value_calculator_disabled_any(
        (pool, lsd, stake_pool_addr, stake_pool, initial_svc_addr, new_balance) in
            (any_pool_state(GenPoolStateArgs {
                bools: PoolStateBools(NewPoolStateBoolsBuilder::start()
                .with_is_disabled(true)
                .with_is_rebalancing(false)
                .build().0.map(|x| Some(Just(x).boxed()))),
                ..Default::default()
            }),
            any_normal_pk(),
            any::<u64>(),
        ).prop_flat_map(
                |(pool, mint_addr, spl_lamports)| (
                    Just(pool),
                    any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != mint_addr),
                    any_spl_stake_pool(GenStakePoolArgs {
                        pool_mint: Some(Just(mint_addr).boxed()),
                        u64s: SplStakePoolU64s(NewSplStakePoolU64sBuilder::start()
                            .with_last_update_epoch(Just(0).boxed()) // mollusk clock defaults to epoch 0
                            .with_total_lamports(Just(spl_lamports).boxed())
                            .with_pool_token_supply((spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed())
                            .build().0.map(Some)),
                        ..Default::default()
                    }),
                    any_lst_state(GenLstStateArgs {
                        sol_value: Some((0..=pool.total_sol_value).boxed()),
                        pks: LstStatePks(NewLstStatePksBuilder::start()
                        .with_mint(mint_addr)
                        .with_sol_value_calculator(sanctum_spl_multi::ID)
                        .build().0.map(|x| Some(Just(x).boxed()))),
                        ..Default::default()
                    }),
                )
            ).prop_flat_map(
                |(pool, stake_pool_addr, stake_pool, lsd)| (
                    Just(pool),
                    Just(lsd),
                    Just(stake_pool_addr),
                    Just(stake_pool),
                    any_normal_pk(),
                    0..=max_sol_val_no_overflow(pool.total_sol_value, lsd.lst_state.sol_value) / MAX_LAMPORTS_OVER_SUPPLY,
                )
            ),
        lsl in any_lst_state_list(Default::default(), 0..=MAX_LST_STATES),
    ) {
        set_sol_value_calculator_proptest(pool, lsl, lsd, pool.admin, *SvcAgTy::SanctumSplMulti(()).svc_program_id(), SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }), initial_svc_addr, new_balance, [
            (lsd.lst_state.mint.into(), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            (Pubkey::new_from_array(stake_pool_addr), mock_spl_stake_pool(stake_pool, sanctum_spl_multi::POOL_PROG_ID.into())),
        ], Some(TestErrorType::PoolDisabled)).unwrap();
    }
}

proptest! {
    #[test]
    fn set_sol_value_calculator_wsol_any(
        (pool, wsol_lsd, initial_svc_addr, new_balance) in
            any_pool_state(GenPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            }).prop_flat_map(
                |pool| (
                    Just(pool),
                    any_wsol_lst_state(GenLstStateArgs {
                        sol_value: Some((0..=pool.total_sol_value).boxed()),
                        ..Default::default()
                    }),
                    any_normal_pk().prop_filter("cannot be eq wsol svc addr", move |x| *x != *SvcAgTy::Wsol(()).svc_program_id()),
                )
            ).prop_flat_map(
                |(pool, wsol_lsd, initial_svc_addr)| (
                    Just(pool),
                    Just(wsol_lsd),
                    Just(initial_svc_addr),
                    0..=max_sol_val_no_overflow(pool.total_sol_value, wsol_lsd.lst_state.sol_value),
                )
            ),
        lsl in any_lst_state_list(Default::default(), 0..=MAX_LST_STATES),
    ) {
        set_sol_value_calculator_proptest(pool, lsl, wsol_lsd, pool.admin, *SvcAgTy::Wsol(()).svc_program_id(), SvcCalcAccsAg::Wsol(WsolCalcAccs), initial_svc_addr, new_balance, [], None).unwrap();
    }
}

proptest! {
    #[test]
    fn set_sol_value_calculator_sanctum_spl_multi_any(
        (pool, lsd, stake_pool_addr, stake_pool, initial_svc_addr, new_balance) in
            (
                any_pool_state(GenPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            }),
            any_normal_pk(),
            any::<u64>(),
            ).prop_flat_map(
                |(pool, mint_addr, spl_lamports)| (
                    Just(pool),
                    any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != mint_addr),
                    any_spl_stake_pool(GenStakePoolArgs {
                        pool_mint: Some(Just(mint_addr).boxed()),
                        u64s: SplStakePoolU64s(NewSplStakePoolU64sBuilder::start()
                            .with_last_update_epoch(Just(0).boxed()) // mollusk clock defaults to epoch 0
                            .with_total_lamports(Just(spl_lamports).boxed())
                            .with_pool_token_supply((spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed())
                            .build().0.map(Some)),
                        ..Default::default()
                    }),
                    any_lst_state(GenLstStateArgs {
                        sol_value: Some((0..=pool.total_sol_value).boxed()),
                        pks: LstStatePks(NewLstStatePksBuilder::start()
                            .with_mint(mint_addr)
                            .with_sol_value_calculator(sanctum_spl_multi::ID)
                            .build().0.map(|x| Some(Just(x).boxed()))),
                        ..Default::default()
                    }),
                    any_normal_pk().prop_filter("cannot be eq sanctum spl multi svc addr", move |x| *x != *SvcAgTy::SanctumSplMulti(()).svc_program_id()),
                )
            ).prop_flat_map(
                |(pool, stake_pool_addr, stake_pool, lsd, initial_svc_addr)| (
                    Just(pool),
                    Just(lsd),
                    Just(stake_pool_addr),
                    Just(stake_pool),
                    Just(initial_svc_addr),
                    0..=max_sol_val_no_overflow(pool.total_sol_value, lsd.lst_state.sol_value) / MAX_LAMPORTS_OVER_SUPPLY,
                )
            ),
        lsl in any_lst_state_list(Default::default(), 0..=MAX_LST_STATES),
    ) {
        set_sol_value_calculator_proptest(pool, lsl, lsd, pool.admin, *SvcAgTy::SanctumSplMulti(()).svc_program_id(), SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }), initial_svc_addr, new_balance, [
            (lsd.lst_state.mint.into(), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            (Pubkey::new_from_array(stake_pool_addr), mock_spl_stake_pool(stake_pool, sanctum_spl_multi::POOL_PROG_ID.into())),
        ], None).unwrap();
    }
}

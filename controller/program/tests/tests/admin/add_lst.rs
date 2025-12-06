use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStateV2},
    err::Inf1CtlErr,
    instructions::admin::add_lst::{
        AddLstIxData, AddLstIxKeysOwned, NewAddLstIxAccsBuilder, ADD_LST_IX_IS_SIGNER,
        ADD_LST_IX_IS_WRITER,
    },
    keys::{
        ATOKEN_ID, LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID, SYS_PROG_ID, TOKENKEG_ID,
    },
    program_err::Inf1CtlCustomProgErr,
    typedefs::lst_state::LstState,
    ID,
};
use inf1_svc_ag_core::SvcAgTy;
use inf1_test_utils::{
    acc_bef_aft, any_lst_state_list, any_normal_pk, any_pool_state_v2, assert_diffs_lst_state_list,
    assert_jiminy_prog_err, find_pool_reserves_ata, find_protocol_fee_accumulator_ata,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, lst_state_list_account, mock_mint,
    mock_token_acc, mollusk_exec, pool_state_v2_account, pool_state_v2_u8_bools_normal_strat,
    raw_mint, raw_token_acc, silence_mollusk_logs, AccountMap, LstStateListChanges,
    LstStateListData, PoolStateV2FtaStrat,
};

use jiminy_cpi::program_error::INVALID_ARGUMENT;

use proptest::{prelude::*, test_runner::TestCaseResult};

use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn add_lst_ix_keys_owned(
    admin: &[u8; 32],
    payer: &[u8; 32],
    mint: &[u8; 32],
    token_program: &[u8; 32],
    sol_value_calculator: &[u8; 32],
) -> AddLstIxKeysOwned {
    let (pool_reserves, _) = find_pool_reserves_ata(token_program, mint);
    let (protocol_fee_accumulator, _) = find_protocol_fee_accumulator_ata(token_program, mint);

    NewAddLstIxAccsBuilder::start()
        .with_admin(*admin)
        .with_payer(*payer)
        .with_lst_mint(*mint)
        .with_pool_reserves(pool_reserves.to_bytes())
        .with_protocol_fee_accumulator(protocol_fee_accumulator.to_bytes())
        .with_protocol_fee_accumulator_auth(PROTOCOL_FEE_ID)
        .with_sol_value_calculator(*sol_value_calculator)
        .with_pool_state(POOL_STATE_ID)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_associated_token_program(ATOKEN_ID)
        .with_system_program(SYS_PROG_ID)
        .with_lst_token_program(*token_program)
        .build()
}

fn add_lst_ix(keys: &AddLstIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        ADD_LST_IX_IS_SIGNER.0.iter(),
        ADD_LST_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: AddLstIxData::as_buf().into(),
    }
}

fn add_lst_fixtures_accounts_opt(keys: &AddLstIxKeysOwned) -> AccountMap {
    fixtures_accounts_opt_cloned(keys.0.iter().copied())
}

fn assert_correct_add(
    bef: &AccountMap,
    aft: &AccountMap,
    mint: &[u8; 32],
    token_program: &[u8; 32],
    expected_sol_value_calculator: &[u8; 32],
) {
    let (_, pool_reserves_bump) = find_pool_reserves_ata(token_program, mint);
    let (_, protocol_fee_accumulator_bump) = find_protocol_fee_accumulator_ata(token_program, mint);

    let lst_state_lists = acc_bef_aft(&Pubkey::new_from_array(LST_STATE_LIST_ID), bef, aft);

    let [lst_state_list_bef, lst_state_list_aft]: [Vec<_>; 2] =
        lst_state_lists.each_ref().map(|a| {
            LstStatePackedList::of_acc_data(&a.data)
                .unwrap()
                .0
                .iter()
                .map(|x| x.into_lst_state())
                .collect()
        });

    let diffs = LstStateListChanges::new(&lst_state_list_bef)
        .with_push(LstState {
            is_input_disabled: 0,
            pool_reserves_bump,
            protocol_fee_accumulator_bump,
            padding: [0u8; 5],
            sol_value: 0,
            mint: *mint,
            sol_value_calculator: *expected_sol_value_calculator,
        })
        .build();

    assert_diffs_lst_state_list(&diffs, &lst_state_list_bef, &lst_state_list_aft);
}

// TODO: pool state fixture no longer applicable with
// v2 upgrade.
// #[test]
// fn add_lst_jitosol_fixture() {}

enum TestErrorType {
    Unauthorized,
    PoolRebalancing,
    PoolDisabled,
    DuplicateLst,
    NonExecSvc,
}

const MAX_LST_STATES: usize = 10;

#[allow(clippy::too_many_arguments)]
fn add_lst_proptest(
    pool: PoolStateV2,
    lsl: LstStateListData,
    admin: [u8; 32],
    payer: [u8; 32],
    mint: [u8; 32],
    token_program: [u8; 32],
    sol_value_calculator: [u8; 32],
    additional_accounts: impl IntoIterator<Item = (Pubkey, Account)>,
    error_type: Option<TestErrorType>,
) -> TestCaseResult {
    silence_mollusk_logs();

    let LstStateListData { lst_state_list, .. } = lsl;

    let keys = add_lst_ix_keys_owned(&admin, &payer, &mint, &token_program, &sol_value_calculator);

    let ix = add_lst_ix(&keys);
    let mut accounts = add_lst_fixtures_accounts_opt(&keys);

    // Common inserts
    accounts.insert(
        LST_STATE_LIST_ID.into(),
        lst_state_list_account(lst_state_list),
    );
    accounts.insert(POOL_STATE_ID.into(), pool_state_v2_account(pool));
    accounts.insert(
        Pubkey::new_from_array(admin),
        Account {
            ..Default::default()
        },
    );
    accounts.insert(
        Pubkey::new_from_array(payer),
        Account {
            lamports: u64::MAX,
            ..Default::default()
        },
    );
    accounts.insert(
        Pubkey::new_from_array(PROTOCOL_FEE_ID),
        Account {
            ..Default::default()
        },
    );

    let (pool_reserves_addr, _) = find_pool_reserves_ata(&token_program, &mint);
    let (protocol_fee_accumulator_addr, _) =
        find_protocol_fee_accumulator_ata(&token_program, &mint);

    accounts.insert(
        pool_reserves_addr,
        mock_token_acc(raw_token_acc(mint, POOL_STATE_ID, 0)),
    );

    accounts.insert(
        protocol_fee_accumulator_addr,
        mock_token_acc(raw_token_acc(mint, PROTOCOL_FEE_ID, 0)),
    );

    // Additional test-specific inserts
    for (pk, acc) in additional_accounts {
        accounts.insert(pk, acc);
    }

    let result = SVM.with(|svm| mollusk_exec(svm, &[ix], &accounts));

    if let Some(error_type) = error_type {
        let err = result.unwrap_err();
        match error_type {
            TestErrorType::Unauthorized => {
                assert_jiminy_prog_err(&err, INVALID_ARGUMENT);
            }
            TestErrorType::DuplicateLst => {
                assert_jiminy_prog_err(&err, Inf1CtlCustomProgErr(Inf1CtlErr::DuplicateLst));
            }
            TestErrorType::PoolRebalancing => {
                assert_jiminy_prog_err(&err, Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing));
            }
            TestErrorType::PoolDisabled => {
                assert_jiminy_prog_err(&err, Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled));
            }
            TestErrorType::NonExecSvc => {
                assert_jiminy_prog_err(
                    &err,
                    Inf1CtlCustomProgErr(Inf1CtlErr::FaultySolValueCalculator),
                );
            }
        }
    } else {
        let resulting_accounts = result.unwrap().resulting_accounts;
        assert_correct_add(
            &accounts,
            &resulting_accounts,
            &mint,
            &token_program,
            &sol_value_calculator,
        );
    }

    Ok(())
}

fn add_lst_correct_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, [u8; 32], [u8; 32])> {
    (any_normal_pk(), any_normal_pk()).prop_flat_map(|(payer, mint)| {
        (
            any_pool_state_v2(PoolStateV2FtaStrat {
                u8_bools: pool_state_v2_u8_bools_normal_strat(),
                ..Default::default()
            })
            .prop_filter("admin cannot be system program", |pool| {
                pool.admin != SYS_PROG_ID
            }),
            any_lst_state_list(Default::default(), None, 0..=0)
                .prop_filter("mint must not be in list", move |lsl| {
                    !lsl.all_pool_reserves.contains_key(&mint)
                }),
            Just(payer),
            Just(mint),
        )
    })
}

proptest! {
    #[test]
    fn add_lst_any(
        (pool, lsl, payer, mint) in add_lst_correct_strat(),
    ) {
        add_lst_proptest(
            pool,
            lsl,
            pool.admin,
            payer,
            mint,
            TOKENKEG_ID,
            *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
            [
                (Pubkey::new_from_array(mint), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            ],
            None,
        ).unwrap();
    }
}

fn add_lst_unauthorized_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, [u8; 32], [u8; 32], [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
        any_normal_pk(),
        any_normal_pk(),
    )
        .prop_flat_map(|(pool, payer, mint)| {
            (
                Just(pool),
                any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES)
                    .prop_filter("mint must not be in list", move |lsl| {
                        !lsl.all_pool_reserves.contains_key(&mint)
                    }),
                Just(payer),
                any_normal_pk().prop_filter("cannot be eq admin", move |x| *x != pool.admin),
                Just(mint),
            )
        })
}

proptest! {
    #[test]
    fn add_lst_unauthorized_any(
        (pool, lsl, payer, non_admin, mint) in add_lst_unauthorized_strat(),
    ) {
        add_lst_proptest(
            pool,
            lsl,
            non_admin,
            payer,
            mint,
            TOKENKEG_ID,
            *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
            [
                (Pubkey::new_from_array(mint), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            ],
            Some(TestErrorType::Unauthorized),
        ).unwrap();
    }
}

fn add_lst_rebalancing_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, [u8; 32], [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_rebalancing(Some(Just(true).boxed())),
            ..Default::default()
        }),
        any_normal_pk(),
        any_normal_pk(),
    )
        .prop_flat_map(|(pool, payer, mint)| {
            (
                Just(pool),
                any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES)
                    .prop_filter("mint must not be in list", move |lsl| {
                        !lsl.all_pool_reserves.contains_key(&mint)
                    }),
                Just(payer),
                Just(mint),
            )
        })
}

proptest! {
    #[test]
    fn add_lst_rebalancing_any(
        (pool, lsl, payer, mint) in add_lst_rebalancing_strat(),
    ) {
        add_lst_proptest(
            pool,
            lsl,
            pool.admin,
            payer,
            mint,
            TOKENKEG_ID,
            *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
            [
                (Pubkey::new_from_array(mint), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            ],
            Some(TestErrorType::PoolRebalancing),
        ).unwrap();
    }
}

fn add_lst_disabled_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, [u8; 32], [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_disabled(Some(Just(true).boxed())),
            ..Default::default()
        }),
        any_normal_pk(),
        any_normal_pk(),
    )
        .prop_flat_map(|(pool, payer, mint)| {
            (
                Just(pool),
                any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES)
                    .prop_filter("mint must not be in list", move |lsl| {
                        !lsl.all_pool_reserves.contains_key(&mint)
                    }),
                Just(payer),
                Just(mint),
            )
        })
}

proptest! {
    #[test]
    fn add_lst_disabled_any(
        (pool, lsl, payer, mint) in add_lst_disabled_strat(),
    ) {
        add_lst_proptest(
            pool,
            lsl,
            pool.admin,
            payer,
            mint,
            TOKENKEG_ID,
            *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
            [
                (Pubkey::new_from_array(mint), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            ],
            Some(TestErrorType::PoolDisabled),
        ).unwrap();
    }
}

fn add_lst_duplicate_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, [u8; 32], [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
        any_lst_state_list(Default::default(), None, 1..=MAX_LST_STATES),
    )
        .prop_flat_map(|(pool, lsl)| {
            let existing_mint = *lsl.all_pool_reserves.keys().next().unwrap();
            (Just(pool), Just(lsl), any_normal_pk(), Just(existing_mint))
        })
}

proptest! {
    #[test]
    fn add_lst_duplicate_any(
        (pool, lsl, payer, existing_mint) in add_lst_duplicate_strat(),
    ) {
        add_lst_proptest(
            pool,
            lsl,
            pool.admin,
            payer,
            existing_mint,
            TOKENKEG_ID,
            *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
            [
                (Pubkey::new_from_array(existing_mint), mock_mint(raw_mint(None, None, u64::MAX, 9))),
            ],
            Some(TestErrorType::DuplicateLst),
        ).unwrap();
    }
}

fn add_lst_non_exec_svc_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, [u8; 32], [u8; 32], [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
        any_normal_pk(),
        any_normal_pk(),
        any_normal_pk(),
    )
        .prop_flat_map(|(pool, payer, mint, sol_value_calculator)| {
            (
                Just(pool),
                any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES)
                    .prop_filter("mint must not be in list", move |lsl| {
                        !lsl.all_pool_reserves.contains_key(&mint)
                    }),
                Just(payer),
                Just(mint),
                Just(sol_value_calculator),
            )
        })
}

proptest! {
    #[test]
    fn add_lst_non_exec_svc_any(
        (pool, lsl, payer, mint, sol_value_calculator) in add_lst_non_exec_svc_strat(),
    ) {
        add_lst_proptest(
            pool,
            lsl,
            pool.admin,
            payer,
            mint,
            TOKENKEG_ID,
            sol_value_calculator,
            [
                (Pubkey::new_from_array(mint), mock_mint(raw_mint(None, None, u64::MAX, 9))),
                (Pubkey::new_from_array(sol_value_calculator), Account {
                    executable: false,
                    ..Default::default()
                }),
            ],
            Some(TestErrorType::NonExecSvc),
        ).unwrap();
    }
}

// TODO: add test for checking u32::MAX limit

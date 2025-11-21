use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStateV2},
    err::Inf1CtlErr,
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxData, RemoveLstIxKeysOwned, REMOVE_LST_IX_IS_SIGNER,
        REMOVE_LST_IX_IS_WRITER,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID, SYS_PROG_ID, TOKENKEG_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_lst_state_list, any_normal_pk, any_pool_state_v2, assert_diffs_lst_state_list,
    assert_jiminy_prog_err, find_pool_reserves_ata, find_protocol_fee_accumulator_ata,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, lst_state_list_account, mock_mint,
    mock_token_acc, mollusk_exec, pool_state_v2_account, pool_state_v2_u8_bools_normal_strat,
    raw_mint, raw_token_acc, silence_mollusk_logs, AccountMap, AnyLstStateArgs,
    LstStateListChanges, LstStateListData, PoolStateV2FtaStrat,
};

use jiminy_cpi::program_error::INVALID_ARGUMENT;

use proptest::{prelude::*, test_runner::TestCaseResult};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn remove_lst_ix_keys_owned(
    admin: &[u8; 32],
    refund_rent_to: &[u8; 32],
    mint: &[u8; 32],
    token_program: &[u8; 32],
) -> RemoveLstIxKeysOwned {
    let (pool_reserves, _) = find_pool_reserves_ata(token_program, mint);
    let (protocol_fee_accumulator, _) = find_protocol_fee_accumulator_ata(token_program, mint);

    NewRemoveLstIxAccsBuilder::start()
        .with_admin(*admin)
        .with_refund_rent_to(*refund_rent_to)
        .with_lst_mint(*mint)
        .with_pool_reserves(pool_reserves.to_bytes())
        .with_protocol_fee_accumulator(protocol_fee_accumulator.to_bytes())
        .with_protocol_fee_accumulator_auth(PROTOCOL_FEE_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_lst_token_program(*token_program)
        .build()
}

fn remove_lst_ix(keys: &RemoveLstIxKeysOwned, lst_idx: u32) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        REMOVE_LST_IX_IS_SIGNER.0.iter(),
        REMOVE_LST_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: RemoveLstIxData::new(lst_idx).as_buf().into(),
    }
}

fn remove_lst_fixtures_accounts_opt(keys: &RemoveLstIxKeysOwned) -> AccountMap {
    fixtures_accounts_opt_cloned(keys.0.iter().copied())
}

fn assert_correct_remove(bef: &AccountMap, aft: &AccountMap, mint: &[u8; 32]) {
    let lst_state_lists = acc_bef_aft(&Pubkey::new_from_array(LST_STATE_LIST_ID), bef, aft);
    let [_, lst_state_list_acc_aft] = lst_state_lists;

    let [lst_state_list_bef, lst_state_list_aft]: [Vec<_>; 2] =
        lst_state_lists.each_ref().map(|a| {
            LstStatePackedList::of_acc_data(&a.data)
                .unwrap()
                .0
                .iter()
                .map(|x| x.into_lst_state())
                .collect()
        });

    let bef_len = lst_state_list_bef.len();

    if bef_len == 1 {
        assert!(
            lst_state_list_acc_aft.data.is_empty()
                && lst_state_list_acc_aft.lamports == 0
                && lst_state_list_acc_aft.owner == Pubkey::new_from_array(SYS_PROG_ID)
        );
    } else {
        let diffs = LstStateListChanges::new(&lst_state_list_bef)
            .with_del_by_mint(mint)
            .build();

        assert_diffs_lst_state_list(&diffs, &lst_state_list_bef, &lst_state_list_aft);
    }
}

// TODO: pool state fixture no longer applicable with
// v2 upgrade.
// #[test]
// fn remove_lst_jupsol_fixture() {}

enum TestErrorType {
    Unauthorized,
    InvalidLstIdx,
    LstStillHasValue,
    PoolRebalancing,
    PoolDisabled,
}

const MAX_LST_STATES: usize = 10;

fn remove_lst_proptest(
    pool: PoolStateV2,
    lsl: LstStateListData,
    admin: [u8; 32],
    refund_rent_to: [u8; 32],
    lst_idx: u32,
    additional_accounts: impl IntoIterator<Item = (Pubkey, Account)>,
    error_type: Option<TestErrorType>,
) -> TestCaseResult {
    silence_mollusk_logs();

    let LstStateListData { lst_state_list, .. } = lsl;

    let lst_state_list_parsed = LstStatePackedList::of_acc_data(&lst_state_list).unwrap();

    // For invalid index tests, we might not have a valid lst_state
    let mint = if let Some(lst_state) = lst_state_list_parsed.0.get(lst_idx as usize) {
        lst_state.into_lst_state().mint
    } else {
        // We still need a mint for the instruction keys
        Pubkey::new_unique().to_bytes()
    };

    let keys = remove_lst_ix_keys_owned(&admin, &refund_rent_to, &mint, &TOKENKEG_ID);

    let ix = remove_lst_ix(&keys, lst_idx);
    let mut accounts = remove_lst_fixtures_accounts_opt(&keys);

    // Common inserts
    accounts.insert(
        LST_STATE_LIST_ID.into(),
        lst_state_list_account(lst_state_list),
    );
    accounts.insert(POOL_STATE_ID.into(), pool_state_v2_account(pool));
    accounts.insert(
        Pubkey::new_from_array(admin),
        Account {
            lamports: u64::MAX,
            ..Default::default()
        },
    );
    accounts.insert(
        Pubkey::new_from_array(refund_rent_to),
        Account {
            ..Default::default()
        },
    );
    accounts.insert(
        Pubkey::new_from_array(mint),
        mock_mint(raw_mint(None, None, u64::MAX, 9)),
    );
    accounts.insert(Pubkey::new_from_array(PROTOCOL_FEE_ID), Account::default());

    let (pool_reserves_addr, _) = find_pool_reserves_ata(&TOKENKEG_ID, &mint);
    let (protocol_fee_accumulator_addr, _) = find_protocol_fee_accumulator_ata(&TOKENKEG_ID, &mint);

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
            TestErrorType::InvalidLstIdx => {
                // Any error is acceptable for invalid index
            }
            TestErrorType::LstStillHasValue => {
                assert_jiminy_prog_err(&err, Inf1CtlCustomProgErr(Inf1CtlErr::LstStillHasValue));
            }
            TestErrorType::PoolRebalancing => {
                assert_jiminy_prog_err(&err, Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing));
            }
            TestErrorType::PoolDisabled => {
                assert_jiminy_prog_err(&err, Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled));
            }
        }
    } else {
        let resulting_accounts = result.unwrap().resulting_accounts;
        assert_correct_remove(&accounts, &resulting_accounts, &mint);
    }

    Ok(())
}

fn remove_lst_correct_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, u32, [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
        any_lst_state_list(
            AnyLstStateArgs {
                sol_value: Some(Just(0).boxed()),
                ..Default::default()
            },
            None,
            1..=MAX_LST_STATES,
        ),
    )
        .prop_flat_map(|(pool, lsl)| {
            let lsl_clone = lsl.clone();
            (
                Just(pool),
                Just(lsl),
                (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                any_normal_pk(),
            )
        })
}

proptest! {
    #[test]
    fn remove_lst_any(
        (pool, lsl, lst_idx, refund_rent_to) in remove_lst_correct_strat(),
    ) {
        remove_lst_proptest(
            pool,
            lsl,
            pool.admin,
            refund_rent_to,
            lst_idx,
            [],
            None,
        ).unwrap();
    }
}

fn remove_lst_unauthorized_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, [u8; 32], u32, [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
        any_lst_state_list(
            AnyLstStateArgs {
                sol_value: Some(Just(0).boxed()),
                ..Default::default()
            },
            None,
            1..=MAX_LST_STATES,
        ),
    )
        .prop_flat_map(|(pool, lsl)| {
            let lsl_clone = lsl.clone();
            (
                Just(pool),
                Just(lsl),
                any_normal_pk().prop_filter("cannot be eq admin", move |x| *x != pool.admin),
                (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                any_normal_pk(),
            )
        })
}

proptest! {
    #[test]
    fn remove_lst_unauthorized_any(
        (pool, lsl, non_admin, lst_idx, refund_rent_to) in remove_lst_unauthorized_strat(),
    ) {
        remove_lst_proptest(
            pool,
            lsl,
            non_admin,
            refund_rent_to,
            lst_idx,
            [],
            Some(TestErrorType::Unauthorized),
        ).unwrap();
    }
}

fn remove_lst_rebalancing_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, u32, [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_rebalancing(Some(Just(true).boxed())),
            ..Default::default()
        }),
        any_lst_state_list(
            AnyLstStateArgs {
                sol_value: Some(Just(0).boxed()),
                ..Default::default()
            },
            None,
            1..=MAX_LST_STATES,
        ),
    )
        .prop_flat_map(|(pool, lsl)| {
            let lsl_clone = lsl.clone();
            (
                Just(pool),
                Just(lsl),
                (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                any_normal_pk(),
            )
        })
}

proptest! {
    #[test]
    fn remove_lst_rebalancing_any(
        (pool, lsl, lst_idx, refund_rent_to) in remove_lst_rebalancing_strat(),
    ) {
        remove_lst_proptest(
            pool,
            lsl,
            pool.admin,
            refund_rent_to,
            lst_idx,
            [],
            Some(TestErrorType::PoolRebalancing),
        ).unwrap();
    }
}

fn remove_lst_disabled_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, u32, [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_disabled(Some(Just(true).boxed())),
            ..Default::default()
        }),
        any_lst_state_list(
            AnyLstStateArgs {
                sol_value: Some(Just(0).boxed()),
                ..Default::default()
            },
            None,
            1..=MAX_LST_STATES,
        ),
    )
        .prop_flat_map(|(pool, lsl)| {
            let lsl_clone = lsl.clone();
            (
                Just(pool),
                Just(lsl),
                (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                any_normal_pk(),
            )
        })
}

proptest! {
    #[test]
    fn remove_lst_disabled_any(
        (pool, lsl, lst_idx, refund_rent_to) in remove_lst_disabled_strat(),
    ) {
        remove_lst_proptest(
            pool,
            lsl,
            pool.admin,
            refund_rent_to,
            lst_idx,
            [],
            Some(TestErrorType::PoolDisabled),
        ).unwrap();
    }
}

fn remove_lst_still_has_value_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, u32, [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
        any_lst_state_list(
            AnyLstStateArgs {
                sol_value: Some((1..u64::MAX).boxed()),
                ..Default::default()
            },
            None,
            1..=MAX_LST_STATES,
        ),
    )
        .prop_flat_map(|(pool, lsl)| {
            let lsl_clone = lsl.clone();
            (
                Just(pool),
                Just(lsl),
                (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                any_normal_pk(),
            )
        })
}

proptest! {
    #[test]
    fn remove_lst_still_has_value_any(
        (pool, lsl, lst_idx, refund_rent_to) in remove_lst_still_has_value_strat(),
    ) {
        remove_lst_proptest(
            pool,
            lsl,
            pool.admin,
            refund_rent_to,
            lst_idx,
            [],
            Some(TestErrorType::LstStillHasValue),
        ).unwrap();
    }
}

fn remove_lst_invalid_lst_idx_strat(
) -> impl Strategy<Value = (PoolStateV2, LstStateListData, u32, [u8; 32])> {
    (
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
        any_lst_state_list(
            AnyLstStateArgs {
                sol_value: Some(Just(0).boxed()),
                ..Default::default()
            },
            None,
            1..=MAX_LST_STATES,
        ),
    )
        .prop_flat_map(|(pool, lsl)| {
            let lsl_clone = lsl.clone();
            (
                Just(pool),
                Just(lsl),
                (lsl_clone.protocol_fee_accumulators.len() as u32..u32::MAX).boxed(),
                any_normal_pk(),
            )
        })
}

proptest! {
    #[test]
    fn remove_lst_invalid_lst_idx_any(
        (pool, lsl, invalid_lst_idx, refund_rent_to) in remove_lst_invalid_lst_idx_strat(),
    ) {
        remove_lst_proptest(
            pool,
            lsl,
            pool.admin,
            refund_rent_to,
            invalid_lst_idx,
            [],
            Some(TestErrorType::InvalidLstIdx),
        ).unwrap();
    }
}

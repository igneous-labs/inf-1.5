use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::{PoolState, PoolStatePacked},
    },
    err::Inf1CtlErr,
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxData, RemoveLstIxKeysOwned, REMOVE_LST_IX_IS_SIGNER,
        REMOVE_LST_IX_IS_WRITER,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID, TOKENKEG_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_lst_state_list, any_normal_pk, any_pool_state, assert_diffs_lst_state_list,
    assert_jiminy_prog_err, find_pool_reserves_ata, find_protocol_fee_accumulator_ata,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, lst_state_list_account, mock_mint,
    mock_token_acc, pool_state_account, raw_mint, raw_token_acc, silence_mollusk_logs,
    upsert_account, AnyLstStateArgs, AnyPoolStateArgs, LstStateListChanges, LstStateListData,
    NewPoolStateBoolsBuilder, PkAccountTup, PoolStateBools, ALL_FIXTURES, JUPSOL_MINT,
};

use jiminy_cpi::program_error::INVALID_ARGUMENT;

use proptest::{prelude::*, test_runner::TestCaseResult};

use mollusk_svm::result::{InstructionResult, ProgramResult};
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

fn remove_lst_fixtures_accounts_opt(keys: &RemoveLstIxKeysOwned) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(keys.0.iter().copied()).collect()
}

fn assert_correct_remove(bef: &[PkAccountTup], aft: &[PkAccountTup], mint: &[u8; 32]) {
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
        assert!(lst_state_list_acc_aft.data.is_empty() && lst_state_list_acc_aft.lamports == 0);
    } else {
        let diffs = LstStateListChanges::new(&lst_state_list_bef)
            .with_del_by_mint(mint)
            .build();

        assert_diffs_lst_state_list(&diffs, &lst_state_list_bef, &lst_state_list_aft);
    }
}

#[test]
fn remove_lst_jupsol_fixture() {
    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_acc = ALL_FIXTURES
        .get(&pool_pk)
        .expect("missing pool state fixture");
    let pool = PoolStatePacked::of_acc_data(&pool_acc.data)
        .unwrap()
        .into_pool_state();

    let admin = pool.admin;
    let token_program = &TOKENKEG_ID;
    let refund_rent_to = Pubkey::new_unique().to_bytes();

    // Find jupSOL in the list to get its index
    let lst_state_list_acc = ALL_FIXTURES
        .get(&Pubkey::new_from_array(LST_STATE_LIST_ID))
        .expect("missing lst state list fixture");
    let lst_state_list = LstStatePackedList::of_acc_data(&lst_state_list_acc.data)
        .unwrap()
        .0;

    let jupsol_idx = lst_state_list
        .iter()
        .position(|packed| {
            let lst = unsafe { packed.as_lst_state() };
            lst.mint == JUPSOL_MINT.to_bytes()
        })
        .expect("jupSOL not found in fixture list");

    let mut lst_state_list_data = lst_state_list_acc.data.clone();
    let lst_state_list = LstStatePackedListMut::of_acc_data(&mut lst_state_list_data).unwrap();
    let lst_state = lst_state_list.0.get_mut(jupsol_idx).unwrap();
    // safety: account data is 8-byte aligned
    unsafe { lst_state.as_lst_state_mut().sol_value = 0 };

    let keys = remove_lst_ix_keys_owned(
        &admin,
        &refund_rent_to,
        &JUPSOL_MINT.to_bytes(),
        token_program,
    );

    let ix = remove_lst_ix(&keys, jupsol_idx as u32);
    let mut accounts = remove_lst_fixtures_accounts_opt(&keys);

    // Upsert the modified LST state list
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(LST_STATE_LIST_ID),
            lst_state_list_account(lst_state_list_data),
        ),
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
            Pubkey::new_from_array(refund_rent_to),
            Account {
                ..Default::default()
            },
        ),
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(PROTOCOL_FEE_ID),
            Account {
                ..Default::default()
            },
        ),
    );

    // Add the ATAs with zero balance
    let (pool_reserves_addr, _) = find_pool_reserves_ata(token_program, &JUPSOL_MINT.to_bytes());
    let (protocol_fee_accumulator_addr, _) =
        find_protocol_fee_accumulator_ata(token_program, &JUPSOL_MINT.to_bytes());

    upsert_account(
        &mut accounts,
        (
            pool_reserves_addr,
            mock_token_acc(raw_token_acc(JUPSOL_MINT.to_bytes(), POOL_STATE_ID, 0)),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            protocol_fee_accumulator_addr,
            mock_token_acc(raw_token_acc(JUPSOL_MINT.to_bytes(), PROTOCOL_FEE_ID, 0)),
        ),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_remove(&accounts, &resulting_accounts, &JUPSOL_MINT.to_bytes());
}

enum TestErrorType {
    Unauthorized,
    InvalidLstIdx,
    LstStillHasValue,
    PoolRebalancing,
    PoolDisabled,
}

const MAX_LST_STATES: usize = 10;

fn remove_lst_proptest(
    pool: PoolState,
    lsl: LstStateListData,
    admin: [u8; 32],
    refund_rent_to: [u8; 32],
    lst_idx: u32,
    additional_accounts: impl IntoIterator<Item = PkAccountTup>,
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

    // Common upserts
    upsert_account(
        &mut accounts,
        (
            LST_STATE_LIST_ID.into(),
            lst_state_list_account(lst_state_list),
        ),
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
            Pubkey::new_from_array(refund_rent_to),
            Account {
                ..Default::default()
            },
        ),
    );
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(mint),
            mock_mint(raw_mint(None, None, u64::MAX, 9)),
        ),
    );
    upsert_account(
        &mut accounts,
        (Pubkey::new_from_array(PROTOCOL_FEE_ID), Account::default()),
    );

    let (pool_reserves_addr, _) = find_pool_reserves_ata(&TOKENKEG_ID, &mint);
    let (protocol_fee_accumulator_addr, _) = find_protocol_fee_accumulator_ata(&TOKENKEG_ID, &mint);

    upsert_account(
        &mut accounts,
        (
            pool_reserves_addr,
            mock_token_acc(raw_token_acc(mint, POOL_STATE_ID, 0)),
        ),
    );
    upsert_account(
        &mut accounts,
        (
            protocol_fee_accumulator_addr,
            mock_token_acc(raw_token_acc(mint, PROTOCOL_FEE_ID, 0)),
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
            TestErrorType::InvalidLstIdx => {
                prop_assert_ne!(program_result, ProgramResult::Success);
            }
            TestErrorType::LstStillHasValue => {
                assert_jiminy_prog_err(
                    &program_result,
                    Inf1CtlCustomProgErr(Inf1CtlErr::LstStillHasValue),
                );
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
        assert_correct_remove(&accounts, &resulting_accounts, &mint);
    }

    Ok(())
}

proptest! {
    #[test]
    fn remove_lst_any(
        (pool, lsl, lst_idx, refund_rent_to) in
            any_pool_state(AnyPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            })
            .prop_flat_map(|pool| {
                (
                    Just(pool),
                    any_lst_state_list(
                        AnyLstStateArgs {
                            sol_value: Some(Just(0).boxed()),
                            ..Default::default()
                        },
                        None, 1..=MAX_LST_STATES).prop_filter("list must not be empty", |lsl| !lsl.lst_state_list.is_empty()),
                )
            })
            .prop_flat_map(|(pool, lsl)| {
                let lsl_clone = lsl.clone();
                (
                    Just(pool),
                    Just(lsl),
                    (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                    any_normal_pk(),
                )
            })
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

proptest! {
    #[test]
    fn remove_lst_unauthorized_any(
        (pool, lsl, non_admin, lst_idx, refund_rent_to) in
            any_pool_state(AnyPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            })
            .prop_flat_map(|pool| {
                (
                    Just(pool),
                    any_lst_state_list(
                        AnyLstStateArgs {
                            sol_value: Some(Just(0).boxed()),
                            ..Default::default()
                        },
                        None, 1..=MAX_LST_STATES).prop_filter("list must not be empty", |lsl| !lsl.lst_state_list.is_empty()),
                )
            })
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

proptest! {
    #[test]
    fn remove_lst_rebalancing_any(
        (pool, lsl, lst_idx, refund_rent_to) in
            any_pool_state(AnyPoolStateArgs {
                bools: PoolStateBools(NewPoolStateBoolsBuilder::start()
                .with_is_disabled(false)
                .with_is_rebalancing(true)
                .build().0.map(|x| Some(Just(x).boxed()))),
                ..Default::default()
            })
            .prop_flat_map(|pool| {
                (
                    Just(pool),
                    any_lst_state_list(
                        AnyLstStateArgs {
                            sol_value: Some(Just(0).boxed()),
                            ..Default::default()
                        },
                        None, 1..=MAX_LST_STATES).prop_filter("list must not be empty", |lsl| !lsl.lst_state_list.is_empty()),
                )
            })
            .prop_flat_map(|(pool, lsl)| {
                let lsl_clone = lsl.clone();
                (
                    Just(pool),
                    Just(lsl),
                    (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                    any_normal_pk(),
                )
            })
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

proptest! {
    #[test]
    fn remove_lst_disabled_any(
        (pool, lsl, lst_idx, refund_rent_to) in
            any_pool_state(AnyPoolStateArgs {
                bools: PoolStateBools(NewPoolStateBoolsBuilder::start()
                .with_is_disabled(true)
                .with_is_rebalancing(false)
                .build().0.map(|x| Some(Just(x).boxed()))),
                ..Default::default()
            })
            .prop_flat_map(|pool| {
                (
                    Just(pool),
                    any_lst_state_list(
                        AnyLstStateArgs {
                            sol_value: Some(Just(0).boxed()),
                            ..Default::default()
                        },
                        None, 1..=MAX_LST_STATES).prop_filter("list must not be empty", |lsl| !lsl.lst_state_list.is_empty()),
                )
            })
            .prop_flat_map(|(pool, lsl)| {
                let lsl_clone = lsl.clone();
                (
                    Just(pool),
                    Just(lsl),
                    (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                    any_normal_pk(),
                )
            })
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

proptest! {
    #[test]
    fn remove_lst_still_has_value_any(
        (pool, lsl, lst_idx, refund_rent_to) in
            any_pool_state(AnyPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            })
            .prop_flat_map(|pool| {
                (
                    Just(pool),
                    any_lst_state_list(
                        AnyLstStateArgs {
                            sol_value: Some((1..u64::MAX).boxed()),
                            ..Default::default()
                        },
                        None, 1..=MAX_LST_STATES).prop_filter("list must not be empty", |lsl| !lsl.lst_state_list.is_empty()),
                )
            })
            .prop_flat_map(|(pool, lsl)| {
                let lsl_clone = lsl.clone();
                (
                    Just(pool),
                    Just(lsl),
                    (0..lsl_clone.protocol_fee_accumulators.len() as u32).boxed(),
                    any_normal_pk(),
                )
            })
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

proptest! {
    #[test]
    fn remove_lst_invalid_lst_idx_any(
        (pool, lsl, invalid_lst_idx, refund_rent_to) in
            any_pool_state(AnyPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            })
            .prop_flat_map(|pool| {
                (
                    Just(pool),
                    any_lst_state_list(
                        AnyLstStateArgs {
                            sol_value: Some(Just(0).boxed()),
                            ..Default::default()
                        },
                        None, 1..=MAX_LST_STATES).prop_filter("list must not be empty", |lsl| !lsl.lst_state_list.is_empty()),
                )
            })
            .prop_flat_map(|(pool, lsl)| {
                let lsl_clone = lsl.clone();
                (
                    Just(pool),
                    Just(lsl),
                    (lsl_clone.protocol_fee_accumulators.len() as u32..u32::MAX).boxed(),
                    any_normal_pk(),
                )
            })
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

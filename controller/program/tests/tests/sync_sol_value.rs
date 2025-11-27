use expect_test::{expect, Expect};
use inf1_core::instructions::sync_sol_value::{
    sync_sol_value_ix_is_signer, sync_sol_value_ix_is_writer, sync_sol_value_ix_keys_owned,
    SyncSolValueIxAccs,
};
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolStateV2Packed, PoolStateV2U64s},
    },
    instructions::sync_sol_value::{
        NewSyncSolValueIxPreAccsBuilder, SyncSolValueIxData, SyncSolValueIxPreAccs,
        SyncSolValueIxPreKeysOwned, SYNC_SOL_VALUE_IX_PRE_ACCS_IDX_LST_MINT,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    ID,
};
use inf1_svc_ag_core::{
    inf1_svc_generic::accounts::state::State,
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_spl_core::{
        instructions::sol_val_calc::SanctumSplMultiCalcAccs, keys::sanctum_spl_multi,
    },
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs,
    instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, any_lst_state, any_lst_state_list, any_normal_pk, any_pool_state_ver,
    any_spl_stake_pool, any_wsol_lst_state, assert_diffs_lst_state_list,
    assert_diffs_pool_state_mm, assert_jiminy_prog_err, find_pool_reserves_ata,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, lst_state_list_account, mock_mint,
    mock_prog_acc, mock_token_acc, mollusk_exec, pool_state_v2_u8_bools_normal_strat, raw_mint,
    raw_token_acc, silence_mollusk_logs, svc_accs, AccountMap, AnyLstStateArgs, AnyPoolStateArgs,
    Diff, DiffLstStateArgs, DiffsPoolStateV2, GenStakePoolArgs, LstStateListChanges, LstStatePks,
    NewLstStatePksBuilder, NewSplStakePoolU64sBuilder, PoolStateBools, PoolStateV2FtaStrat,
    ProgramDataAddr, SplStakePoolU64s, SplSvcAccParams, SvcAccParamsAg, VerPoolState,
    JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT, WSOL_MINT,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::{
    result::{InstructionResult, ProgramResult},
    Mollusk,
};
use proptest::prelude::*;
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{
    jupsol_fixtures_svc_suf, max_sol_val_no_overflow, MAX_LAMPORTS_OVER_SUPPLY, MAX_LST_STATES, SVM,
};

type SyncSolValueKeysBuilder =
    SyncSolValueIxAccs<[u8; 32], SyncSolValueIxPreKeysOwned, SvcCalcAccsAg>;

fn sync_sol_value_ix_pre_keys_owned(
    token_program: &[u8; 32],
    mint: [u8; 32],
) -> SyncSolValueIxPreKeysOwned {
    NewSyncSolValueIxPreAccsBuilder::start()
        .with_lst_mint(mint)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_pool_reserves(find_pool_reserves_ata(token_program, &mint).0.to_bytes())
        .build()
}

fn sync_sol_value_ix(builder: &SyncSolValueKeysBuilder, lst_idx: u32) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        sync_sol_value_ix_keys_owned(builder).seq(),
        sync_sol_value_ix_is_signer(builder).seq(),
        sync_sol_value_ix_is_writer(builder).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SyncSolValueIxData::new(lst_idx).as_buf().into(),
    }
}

fn sync_sol_value_fixtures_accounts_opt(builder: &SyncSolValueKeysBuilder) -> AccountMap {
    fixtures_accounts_opt_cloned(sync_sol_value_ix_keys_owned(builder).seq().copied())
}

/// Returns `new_sol_value - old_sol_value`
fn assert_correct_sync(
    bef: &AccountMap,
    aft: &AccountMap,
    mint: &[u8; 32],
    migration_slot: u64,
) -> i128 {
    let [[pool_bef, pool_aft], lst_state_lists] = [POOL_STATE_ID, LST_STATE_LIST_ID]
        .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), bef, aft));

    let [lst_state_list_bef, lst_state_list_aft]: [Vec<_>; 2] =
        lst_state_lists.each_ref().map(|a| {
            LstStatePackedList::of_acc_data(&a.data)
                .unwrap()
                .0
                .iter()
                .map(|x| x.into_lst_state())
                .collect()
        });
    let lst_state_i = lst_state_list_bef
        .iter()
        .position(|s| s.mint == *mint)
        .unwrap();
    let diffs = LstStateListChanges::new(&lst_state_list_bef)
        .with_diff_by_mint(
            mint,
            DiffLstStateArgs {
                // dont care abt change here,
                // only assert pool total sol value
                // changed by the same amount below
                sol_value: Diff::Pass,
                ..Default::default()
            },
        )
        .build();
    assert_diffs_lst_state_list(&diffs, &lst_state_list_bef, &lst_state_list_aft);

    let [lst_state_bef, lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l[lst_state_i]);
    let expected_delta = i128::from(lst_state_aft.sol_value) - i128::from(lst_state_bef.sol_value);

    let pool_bef = VerPoolState::from_acc_data(&pool_bef.data);
    let pool_aft = PoolStateV2Packed::of_acc_data(&pool_aft.data)
        .unwrap()
        .into_pool_state_v2();

    let expected_total_sol_value =
        u64::try_from(i128::from(pool_bef.total_sol_value()) + expected_delta).unwrap();
    assert_diffs_pool_state_mm(
        DiffsPoolStateV2 {
            u64s: PoolStateV2U64s::default()
                .with_total_sol_value(Diff::Changed(
                    pool_bef.total_sol_value(),
                    expected_total_sol_value,
                ))
                // these 2 fields may change if change of svc
                // results in loss of SOL value
                //
                // TODO: assert correctness of decrease
                .with_withheld_lamports(Diff::Pass)
                .with_protocol_fee_lamports(Diff::Pass),
            ..Default::default()
        },
        &pool_bef,
        &pool_aft,
        migration_slot,
    );

    expected_delta
}

fn assert_correct_sync_snapshot(
    bef: &AccountMap,
    aft: &AccountMap,
    mint: &[u8; 32],
    migration_slot: u64,
    expected_sol_val_delta: Expect,
) {
    let delta = assert_correct_sync(bef, aft, mint, migration_slot);
    expected_sol_val_delta.assert_eq(&delta.to_string());
}

#[test]
fn sync_sol_value_jupsol_fixture() {
    let ix_prefix = sync_sol_value_ix_pre_keys_owned(&TOKENKEG_PROGRAM, JUPSOL_MINT.to_bytes());
    let builder = SyncSolValueKeysBuilder {
        ix_prefix,
        calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        calc: jupsol_fixtures_svc_suf(),
    };
    let ix = sync_sol_value_ix(&builder, JUPSOL_FIXTURE_LST_IDX as u32);
    let accounts = sync_sol_value_fixtures_accounts_opt(&builder);
    let (
        (
            bef,
            InstructionResult {
                program_result,
                resulting_accounts,
                ..
            },
        ),
        migration_slot,
    ) = SVM.with(|svm| (mollusk_exec(svm, &ix, &accounts), svm.sysvars.clock.slot));

    assert_eq!(program_result, ProgramResult::Success);

    let aft: AccountMap = resulting_accounts.into_iter().collect();
    assert_correct_sync_snapshot(
        &bef,
        &aft,
        JUPSOL_MINT.as_array(),
        migration_slot,
        expect!["547883064440"],
    );
}

fn sync_sol_value_test(
    svm: &Mollusk,
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let migration_slot = svm.sysvars.clock.slot;
    let (
        bef,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = mollusk_exec(svm, ix, bef);
    let aft: AccountMap = resulting_accounts.into_iter().collect();

    let mint = ix.accounts[SYNC_SOL_VALUE_IX_PRE_ACCS_IDX_LST_MINT]
        .pubkey
        .as_array();

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_correct_sync(&bef, &aft, mint, migration_slot);
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }
}

#[derive(Debug)]
struct TestParams {
    pool: VerPoolState,
    lst_state_list: Vec<u8>,
    reserves: RawTokenAccount,
    lst_idx: u32,
}

fn prefix_accounts(
    pre: SyncSolValueIxPreKeysOwned,
    TestParams {
        pool,
        lst_state_list,
        reserves,
        ..
    }: TestParams,
) -> AccountMap {
    let pre = SyncSolValueIxPreAccs(pre.0.map(Pubkey::from));
    NewSyncSolValueIxPreAccsBuilder::start()
        .with_pool_state((*pre.pool_state(), pool.into_account()))
        .with_lst_state_list((
            *pre.lst_state_list(),
            lst_state_list_account(lst_state_list),
        ))
        .with_lst_mint((
            *pre.lst_mint(),
            // mint state should not affect instruction at all
            mock_mint(raw_mint(None, None, u64::MAX, 9)),
        ))
        .with_pool_reserves((*pre.pool_reserves(), mock_token_acc(reserves)))
        .build()
        .0
        .into_iter()
        .collect()
}

type SyncSolValueParams = SyncSolValueIxAccs<[u8; 32], SyncSolValueIxPreKeysOwned, SvcAccParamsAg>;

fn sync_sol_value_inp(
    SyncSolValueIxAccs {
        ix_prefix,
        calc_prog,
        calc,
    }: SyncSolValueParams,
    params: TestParams,
) -> (Instruction, AccountMap) {
    let (calc, svc_accounts) = svc_accs(calc);
    (
        sync_sol_value_ix(
            &SyncSolValueIxAccs {
                ix_prefix,
                calc_prog,
                calc,
            },
            params.lst_idx,
        ),
        prefix_accounts(ix_prefix, params)
            .into_iter()
            .chain(core::iter::once((
                calc_prog.into(),
                mock_prog_acc(ProgramDataAddr::Raw(Default::default())), // dont care abt progdata of calc prog
            )))
            .chain(svc_accounts)
            .collect(),
    )
}

fn wsol_correct_strat() -> impl Strategy<Value = (SyncSolValueParams, TestParams)> {
    any_pool_state_ver(
        AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        },
        PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        },
    )
    .prop_flat_map(|pool| {
        (
            Just(pool),
            any_wsol_lst_state(AnyLstStateArgs {
                sol_value: Some((0..=pool.total_sol_value()).boxed()),
                ..Default::default()
            }),
        )
    })
    .prop_flat_map(|(pool, wsol_lsd)| {
        (
            Just(pool),
            Just(wsol_lsd),
            0..=max_sol_val_no_overflow(pool.total_sol_value(), wsol_lsd.lst_state.sol_value),
            any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
        )
    })
    .prop_map(|(pool, wsol_lsd, new_bal, mut lsl)| {
        let lst_idx = lsl.upsert(wsol_lsd).try_into().unwrap();
        (
            SyncSolValueIxAccs {
                ix_prefix: sync_sol_value_ix_pre_keys_owned(
                    &TOKENKEG_PROGRAM,
                    WSOL_MINT.to_bytes(),
                ),
                calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
                calc: SvcAccParamsAg::Wsol(WsolCalcAccs),
            },
            TestParams {
                pool,
                lst_state_list: lsl.lst_state_list,
                reserves: raw_token_acc(WSOL_MINT.to_bytes(), POOL_STATE_ID, new_bal),
                lst_idx,
            },
        )
    })
}

proptest! {
    #[test]
    fn sync_sol_value_wsol_any(
        (ix, bef) in wsol_correct_strat().prop_map(|(a, b)| sync_sol_value_inp(a, b)),
    ) {
        silence_mollusk_logs();
        SVM.with(|svm| {
            sync_sol_value_test(svm, &ix, &bef, None::<ProgramError>);
        });
    }
}

fn sanctum_spl_multi_correct_strat() -> impl Strategy<Value = (SyncSolValueParams, TestParams)> {
    (
        any_pool_state_ver(
            AnyPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            },
            PoolStateV2FtaStrat {
                u8_bools: pool_state_v2_u8_bools_normal_strat(),
                ..Default::default()
            },
        ),
        any_normal_pk(),
        any::<u64>(),
    )
        .prop_flat_map(|(pool, mint_addr, spl_lamports)| {
            (
                Just(pool),
                any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != mint_addr),
                any_spl_stake_pool(GenStakePoolArgs {
                    pool_mint: Some(Just(mint_addr).boxed()),
                    u64s: SplStakePoolU64s(
                        NewSplStakePoolU64sBuilder::start()
                            .with_last_update_epoch(Just(0).boxed()) // mollusk clock defaults to epoch 0
                            .with_total_lamports(Just(spl_lamports).boxed())
                            .with_pool_token_supply(
                                (spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed(),
                            )
                            .build()
                            .0
                            .map(Some),
                    ),
                    ..Default::default()
                }),
                any_lst_state(
                    AnyLstStateArgs {
                        sol_value: Some((0..=pool.total_sol_value()).boxed()),
                        pks: LstStatePks(
                            NewLstStatePksBuilder::start()
                                .with_mint(mint_addr)
                                .with_sol_value_calculator(sanctum_spl_multi::ID)
                                .build()
                                .0
                                .map(|x| Some(Just(x).boxed())),
                        ),
                        ..Default::default()
                    },
                    None,
                ),
            )
        })
        .prop_flat_map(|(pool, stake_pool_addr, stake_pool, lsd)| {
            (
                Just(pool),
                Just(lsd),
                Just(stake_pool_addr),
                Just(stake_pool),
                0..=max_sol_val_no_overflow(pool.total_sol_value(), lsd.lst_state.sol_value)
                    / MAX_LAMPORTS_OVER_SUPPLY,
                any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
            )
        })
        .prop_map(
            |(pool, lsd, stake_pool_addr, stake_pool, new_bal, mut lsl)| {
                let lst_idx = lsl.upsert(lsd).try_into().unwrap();
                (
                    SyncSolValueIxAccs {
                        ix_prefix: sync_sol_value_ix_pre_keys_owned(
                            &TOKENKEG_PROGRAM,
                            lsd.lst_state.mint,
                        ),
                        calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
                        calc: SvcAccParamsAg::SanctumSplMulti((
                            SanctumSplMultiCalcAccs { stake_pool_addr },
                            SplSvcAccParams {
                                pool: stake_pool,
                                gpc_state: State::default(),
                                last_prog_upg_slot: 0,
                            },
                        )),
                    },
                    TestParams {
                        pool,
                        lst_state_list: lsl.lst_state_list,
                        reserves: raw_token_acc(WSOL_MINT.to_bytes(), POOL_STATE_ID, new_bal),
                        lst_idx,
                    },
                )
            },
        )
}

proptest! {
    #[test]
    fn sync_sol_value_sanctum_spl_multi_any(
        (ix, bef) in sanctum_spl_multi_correct_strat().prop_map(|(a, b)| sync_sol_value_inp(a, b)),
    ) {
        silence_mollusk_logs();
        SVM.with(|svm| {
            sync_sol_value_test(svm, &ix, &bef, None::<ProgramError>);
        });
    }
}

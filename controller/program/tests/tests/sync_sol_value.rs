use expect_test::{expect, Expect};
use inf1_core::instructions::sync_sol_value::{
    sync_sol_value_ix_is_signer, sync_sol_value_ix_is_writer, sync_sol_value_ix_keys_owned,
    SyncSolValueIxAccs,
};
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolState, PoolStatePacked},
    },
    instructions::sync_sol_value::{
        NewSyncSolValueIxPreAccsBuilder, SyncSolValueIxData, SyncSolValueIxPreKeysOwned,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    ID,
};
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, any_lst_state_list, any_pool_state, any_wsol_lst_state, find_pool_reserves,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, lst_state_list_account,
    mock_token_acc, pool_state_account, raw_token_acc, silence_mollusk_logs, upsert_account,
    GenLstStateArgs, GenPoolStateArgs, LstStateData, LstStateListData, PkAccountTup,
    PoolStateBools, JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT, WSOL_MINT,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::{prelude::*, test_runner::TestCaseResult};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{jupsol_fixtures_svc_suf, MAX_LST_STATES, SVM};

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
        .with_pool_reserves(find_pool_reserves(token_program, &mint).0.to_bytes())
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

fn sync_sol_value_fixtures_accounts_opt(builder: &SyncSolValueKeysBuilder) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(sync_sol_value_ix_keys_owned(builder).seq().copied()).collect()
}

/// Returns `new_sol_value - old_sol_value`
fn assert_correct_sync(bef: &[PkAccountTup], aft: &[PkAccountTup], mint: &[u8; 32]) -> i128 {
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

    let delta = i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value);
    assert_eq!(
        delta,
        i128::from(lst_state_aft.sol_value) - i128::from(lst_state_bef.sol_value)
    );

    delta
}

fn assert_correct_sync_snapshot(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    mint: &[u8; 32],
    expected_sol_val_delta: Expect,
) {
    let delta = assert_correct_sync(bef, aft, mint);
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

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_sync_snapshot(
        &accounts,
        &resulting_accounts,
        JUPSOL_MINT.as_array(),
        expect!["547883064440"],
    );
}

fn sync_sol_value_wsol_proptest(
    pool: PoolState,
    mut lsl: LstStateListData,
    wsol_lsd: LstStateData,
    new_balance: u64,
) -> TestCaseResult {
    silence_mollusk_logs();
    let wsol_idx = lsl.upsert(wsol_lsd);
    let LstStateListData {
        lst_state_list,
        all_pool_reserves,
        ..
    } = lsl;
    let ix_prefix = sync_sol_value_ix_pre_keys_owned(&TOKENKEG_PROGRAM, WSOL_MINT.to_bytes());
    let builder = SyncSolValueKeysBuilder {
        ix_prefix,
        calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    };
    let ix = sync_sol_value_ix(&builder, wsol_idx as u32);
    let mut accounts = sync_sol_value_fixtures_accounts_opt(&builder);
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
            Pubkey::new_from_array(*all_pool_reserves.get(WSOL_MINT.as_array()).unwrap()),
            mock_token_acc(raw_token_acc(
                WSOL_MINT.to_bytes(),
                POOL_STATE_ID,
                new_balance,
            )),
        ),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    prop_assert_eq!(program_result, ProgramResult::Success);

    assert_correct_sync(&accounts, &resulting_accounts, WSOL_MINT.as_array());

    Ok(())
}

proptest! {
    #[test]
    fn sync_sol_value_wsol_any(
        (pool, wsol_lsd, new_balance) in
            any_pool_state(GenPoolStateArgs {
                bools: PoolStateBools::normal(),
                ..Default::default()
            }).prop_flat_map(
                |pool| (
                    Just(pool),
                    any_wsol_lst_state(GenLstStateArgs { sol_value: Some((0..=pool.total_sol_value).boxed()), ..Default::default() }),
                )
            ).prop_flat_map(
                |(pool, wsol_lsd)| (
                    Just(pool),
                    Just(wsol_lsd),
                    0..=(u64::MAX - (pool.total_sol_value - wsol_lsd.lst_state.sol_value)) // avoid overflow/MathError
                )
            ),
        lsl in any_lst_state_list(Default::default(), 0..=MAX_LST_STATES),
    ) {
        sync_sol_value_wsol_proptest(pool, lsl, wsol_lsd, new_balance).unwrap();
    }
}

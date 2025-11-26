use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed},
    instructions::admin::set_admin::{
        NewSetAdminIxAccsBuilder, SetAdminIxData, SetAdminIxKeysOwned, SET_ADMIN_IX_ACCS_IDX_CURR,
        SET_ADMIN_IX_ACCS_IDX_NEW, SET_ADMIN_IX_IS_SIGNER, SET_ADMIN_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state_v2, assert_diffs_pool_state_v2,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec,
    pool_state_v2_account, silence_mollusk_logs, AccountMap, Diff, DiffsPoolStateV2,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_admin_ix(keys: SetAdminIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_ADMIN_IX_IS_SIGNER.0.iter(),
        SET_ADMIN_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetAdminIxData::as_buf().into(),
    }
}

fn set_admin_test_accs(keys: SetAdminIxKeysOwned, pool: PoolStateV2) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetAdminIxAccsBuilder::start()
        .with_curr(mock_sys_acc(LAMPORTS))
        .with_new(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_v2_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `pool_state.admin` at the end of ix
fn set_admin_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> [u8; 32] {
    let (
        bef,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, ix, bef));
    let aft: AccountMap = resulting_accounts.into_iter().collect();

    let [pool_state_bef, pool_state_aft] =
        acc_bef_aft(&POOL_STATE_ID.into(), &bef, &aft).map(|a| {
            PoolStateV2Packed::of_acc_data(&a.data)
                .unwrap()
                .into_pool_state_v2()
        });

    let curr_admin = pool_state_bef.admin;
    let expected_new_admin = ix.accounts[SET_ADMIN_IX_ACCS_IDX_NEW].pubkey;

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_diffs_pool_state_v2(
                &DiffsPoolStateV2 {
                    addrs: PoolStateV2Addrs::default()
                        .with_admin(Diff::Changed(curr_admin, expected_new_admin.to_bytes())),
                    ..Default::default()
                },
                &pool_state_bef,
                &pool_state_aft,
            );
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }

    pool_state_aft.admin
}

#[test]
fn set_admin_test_correct_basic() {
    let [curr_admin, new_admin] = core::array::from_fn(|i| [u8::try_from(i).unwrap(); 32]);
    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default().with_admin(curr_admin),
        ..Default::default()
    }
    .into_pool_state_v2();
    let keys = NewSetAdminIxAccsBuilder::start()
        .with_new(new_admin)
        .with_curr(curr_admin)
        .with_pool_state(POOL_STATE_ID)
        .build();
    let ret = set_admin_test(
        &set_admin_ix(keys),
        &set_admin_test_accs(keys, pool),
        Option::<ProgramError>::None,
    );
    assert_eq!(ret, new_admin);
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_pool_state_v2(Default::default()))
        .prop_map(|(new_admin, ps)| {
            (
                NewSetAdminIxAccsBuilder::start()
                    .with_new(new_admin)
                    .with_curr(ps.admin)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| (set_admin_ix(k), set_admin_test_accs(k, ps)))
}

proptest! {
    #[test]
    fn set_admin_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        set_admin_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_pool_state_v2(Default::default()))
        .prop_flat_map(|(new_admin, ps)| {
            (
                any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.admin),
                Just(new_admin),
                Just(ps),
            )
        })
        .prop_map(|(wrong_curr_admin, new_admin, ps)| {
            (
                NewSetAdminIxAccsBuilder::start()
                    .with_new(new_admin)
                    .with_curr(wrong_curr_admin)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| (set_admin_ix(k), set_admin_test_accs(k, ps)))
}

proptest! {
    #[test]
    fn set_admin_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_admin_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[SET_ADMIN_IX_ACCS_IDX_CURR].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn set_admin_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_admin_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

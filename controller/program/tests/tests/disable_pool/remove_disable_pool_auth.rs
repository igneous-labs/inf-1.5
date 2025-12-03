use inf1_ctl_jiminy::{
    accounts::{
        disable_pool_authority_list::DisablePoolAuthorityList,
        pool_state::{PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals},
    },
    err::Inf1CtlErr,
    instructions::disable_pool::remove_disable_pool_auth::{
        NewRemoveDisablePoolAuthIxAccsBuilder, RemoveDisablePoolAuthIxData,
        RemoveDisablePoolAuthIxKeysOwned, REMOVE_DISABLE_POOL_AUTH_IX_ACCS_IDX_REMOVE,
        REMOVE_DISABLE_POOL_AUTH_IX_ACCS_IDX_SIGNER, REMOVE_DISABLE_POOL_AUTH_IX_IS_SIGNER,
        REMOVE_DISABLE_POOL_AUTH_IX_IS_WRITER,
    },
    keys::{DISABLE_POOL_AUTHORITY_LIST_ID, POOL_STATE_ID, SYS_PROG_ID},
    program_err::Inf1CtlCustomProgErr,
};
use inf1_test_utils::{
    any_disable_pool_auth_list, any_normal_pk, any_pool_state_v2,
    assert_diffs_disable_pool_auth_list, assert_jiminy_prog_err,
    assert_valid_disable_pool_auth_list, disable_pool_auth_list_account, distinct_idxs, idx_oob,
    keys_signer_writable_to_metas, list_sample_flat_map, mock_sys_acc, mollusk_exec,
    pool_state_v2_account, silence_mollusk_logs, AccountMap, DisablePoolAuthListChanges,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use proptest::{prelude::*, strategy::Union};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{common::SVM, tests::disable_pool::common::MAX_DISABLE_POOL_AUTH_LIST_LEN};

fn remove_disable_pool_auth_ix(keys: RemoveDisablePoolAuthIxKeysOwned, idx: u32) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        REMOVE_DISABLE_POOL_AUTH_IX_IS_SIGNER.0.iter(),
        REMOVE_DISABLE_POOL_AUTH_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: RemoveDisablePoolAuthIxData::new(idx).as_buf().into(),
    }
}

fn remove_disable_pool_auth_test_accs(
    keys: RemoveDisablePoolAuthIxKeysOwned,
    pool: PoolStateV2,
    disable_pool_auth_list: Vec<[u8; 32]>,
) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewRemoveDisablePoolAuthIxAccsBuilder::start()
        .with_signer(mock_sys_acc(LAMPORTS))
        .with_refund_rent_to(mock_sys_acc(LAMPORTS))
        .with_remove(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_v2_account(pool))
        .with_disable_pool_auth_list(disable_pool_auth_list_account(disable_pool_auth_list))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

fn remove_disable_pool_auth_test(
    ix: Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let removed = ix.accounts[REMOVE_DISABLE_POOL_AUTH_IX_ACCS_IDX_REMOVE]
        .pubkey
        .to_bytes();
    let result = SVM.with(|svm| mollusk_exec(svm, &[ix], bef));

    let list_bef = DisablePoolAuthorityList::of_acc_data(
        &bef.get(&DISABLE_POOL_AUTHORITY_LIST_ID.into())
            .unwrap()
            .data,
    )
    .unwrap()
    .0;

    match expected_err {
        None => {
            let resulting_accounts = result.unwrap().resulting_accounts;
            let list_acc_aft = resulting_accounts
                .get(&DISABLE_POOL_AUTHORITY_LIST_ID.into())
                .unwrap();
            let list_aft = DisablePoolAuthorityList::of_acc_data(&list_acc_aft.data)
                .unwrap()
                .0;
            assert_diffs_disable_pool_auth_list(
                DisablePoolAuthListChanges::new(list_bef)
                    .with_del_by_pk(&removed)
                    .build(),
                list_bef,
                list_aft,
            );
            if list_aft.is_empty() {
                assert_eq!(list_acc_aft.owner, SYS_PROG_ID.into());
            }
            assert_valid_disable_pool_auth_list(list_aft);
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
        }
    }
}

#[test]
fn remove_disable_pool_auth_correct_basic() {
    // +69 to avoid using system program [0; 32]
    let [admin, remove] = core::array::from_fn(|i| [u8::try_from(i + 69).unwrap(); 32]);
    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default().with_admin(admin),
        ..Default::default()
    }
    .into_pool_state_v2();
    let keys = NewRemoveDisablePoolAuthIxAccsBuilder::start()
        .with_signer(admin)
        .with_refund_rent_to(admin)
        .with_remove(remove)
        .with_pool_state(POOL_STATE_ID)
        .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
        .build();
    remove_disable_pool_auth_test(
        remove_disable_pool_auth_ix(keys, 0),
        &remove_disable_pool_auth_test_accs(keys, pool, vec![remove]),
        Option::<ProgramError>::None,
    );
}

fn to_inp(
    (k, idx, ps, list): (
        RemoveDisablePoolAuthIxKeysOwned,
        usize,
        PoolStateV2,
        Vec<[u8; 32]>,
    ),
) -> (Instruction, AccountMap) {
    (
        remove_disable_pool_auth_ix(k, idx.try_into().unwrap()),
        remove_disable_pool_auth_test_accs(k, ps, list),
    )
}

/// Set of keys that will result in a successful execution,
/// authorized by the pool admin
fn correct_admin_keys(
    ps: &PoolStateV2,
    refund: [u8; 32],
    remove: [u8; 32],
) -> RemoveDisablePoolAuthIxKeysOwned {
    NewRemoveDisablePoolAuthIxAccsBuilder::start()
        .with_signer(ps.admin)
        .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_refund_rent_to(refund)
        .with_remove(remove)
        .build()
}

fn correct_admin_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_pool_state_v2(Default::default()),
        any_disable_pool_auth_list(1..=MAX_DISABLE_POOL_AUTH_LIST_LEN)
            .prop_flat_map(list_sample_flat_map),
    )
        .prop_map(|(refund, ps, (idx, remove, list))| {
            (correct_admin_keys(&ps, refund, remove), idx, ps, list)
        })
        .prop_map(to_inp)
}

proptest! {
    /// authorized by admin
    #[test]
    fn remove_disable_pool_auth_admin_correct_pt(
        (ix, bef) in correct_admin_strat(),
    ) {
        silence_mollusk_logs();
        remove_disable_pool_auth_test(ix, &bef, Option::<ProgramError>::None);
    }
}

fn correct_remove_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_pool_state_v2(Default::default()),
        any_disable_pool_auth_list(1..=MAX_DISABLE_POOL_AUTH_LIST_LEN)
            .prop_flat_map(list_sample_flat_map),
    )
        .prop_map(|(refund, ps, (idx, remove, list))| {
            (
                correct_admin_keys(&ps, refund, remove).with_signer(remove),
                idx,
                ps,
                list,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    /// authorized by auth being removed
    #[test]
    fn remove_disable_pool_auth_remove_correct_pt(
        (ix, bef) in correct_remove_strat(),
    ) {
        silence_mollusk_logs();
        remove_disable_pool_auth_test(ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_pool_state_v2(Default::default()),
        any_disable_pool_auth_list(1..=MAX_DISABLE_POOL_AUTH_LIST_LEN),
    )
        .prop_flat_map(|(ps, list)| {
            let l = list.clone();
            (
                any_normal_pk(),
                any_normal_pk().prop_filter("", move |pk| *pk != ps.admin && !l.contains(pk)),
                Just(ps),
                list_sample_flat_map(list),
            )
        })
        .prop_map(|(refund, unauthorized, ps, (idx, remove, list))| {
            (
                correct_admin_keys(&ps, refund, remove).with_signer(unauthorized),
                idx,
                ps,
                list,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn remove_disable_pool_auth_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        remove_disable_pool_auth_test(
            ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedDisablePoolAuthoritySigner))
        );
    }
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    Union::new([
        correct_admin_strat().boxed(),
        correct_remove_strat().boxed(),
    ])
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[REMOVE_DISABLE_POOL_AUTH_IX_ACCS_IDX_SIGNER].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn remove_disable_pool_auth_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        remove_disable_pool_auth_test(ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn idx_oob_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_normal_pk(),
        any_pool_state_v2(Default::default()),
        any_disable_pool_auth_list(0..=MAX_DISABLE_POOL_AUTH_LIST_LEN)
            .prop_flat_map(|l| (idx_oob(l.len()), Just(l))),
    )
        .prop_map(|(refund, remove, ps, (oob, list))| {
            (correct_admin_keys(&ps, refund, remove), oob, ps, list)
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn remove_disable_pool_auth_idx_oob_pt(
        (ix, bef) in idx_oob_strat(),
    ) {
        silence_mollusk_logs();
        remove_disable_pool_auth_test(
            ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidDisablePoolAuthorityIndex))
        );
    }
}

fn idx_mismatch_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_pool_state_v2(Default::default()),
        any_disable_pool_auth_list(2..=MAX_DISABLE_POOL_AUTH_LIST_LEN) // need at least 2 for 2 distinct indexes
            .prop_flat_map(|l| (distinct_idxs(l.len()), Just(l))),
    )
        .prop_flat_map(|(refund, ps, ((x, y), list))| {
            let remove = list[x];
            let ak = correct_admin_keys(&ps, refund, remove);
            (
                Union::new([Just(ak), Just(ak.with_signer(remove))]),
                Just(y),
                Just(ps),
                Just(list),
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn remove_disable_pool_auth_idx_mismatch_pt(
        (ix, bef) in idx_mismatch_strat(),
    ) {
        silence_mollusk_logs();
        remove_disable_pool_auth_test(ix, &bef, Some(INVALID_ARGUMENT));
    }
}

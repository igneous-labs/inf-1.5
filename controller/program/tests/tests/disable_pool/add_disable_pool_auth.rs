use inf1_ctl_jiminy::{
    accounts::{disable_pool_authority_list::DisablePoolAuthorityList, pool_state::PoolState},
    err::Inf1CtlErr,
    instructions::disable_pool::add_disable_pool_auth::{
        AddDisablePoolAuthIxData, AddDisablePoolAuthIxKeysOwned,
        NewAddDisablePoolAuthIxAccsBuilder, ADD_DISABLE_POOL_AUTH_IX_ACCS_IDX_ADMIN,
        ADD_DISABLE_POOL_AUTH_IX_ACCS_IDX_NEW, ADD_DISABLE_POOL_AUTH_IX_IS_SIGNER,
        ADD_DISABLE_POOL_AUTH_IX_IS_WRITER,
    },
    keys::{DISABLE_POOL_AUTHORITY_LIST_ID, POOL_STATE_ID, SYS_PROG_ID},
    program_err::Inf1CtlCustomProgErr,
};
use inf1_test_utils::{
    acc_bef_aft, any_disable_pool_auth_list, any_normal_pk, any_pool_state, assert_balanced,
    assert_diffs_disable_pool_auth_list, assert_jiminy_prog_err,
    assert_valid_disable_pool_auth_list, disable_pool_auth_list_account, gen_pool_state,
    keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec_validate, pool_state_account,
    silence_mollusk_logs, AccountMap, DisablePoolAuthListChanges, GenPoolStateArgs, PoolStatePks,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::{
    program::keyed_account_for_system_program,
    result::{Check, InstructionResult, ProgramResult},
};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{common::SVM, tests::disable_pool::common::MAX_DISABLE_POOL_AUTH_LIST_LEN};

fn add_disable_pool_auth_ix(keys: AddDisablePoolAuthIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        ADD_DISABLE_POOL_AUTH_IX_IS_SIGNER.0.iter(),
        ADD_DISABLE_POOL_AUTH_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: AddDisablePoolAuthIxData::as_buf().into(),
    }
}

fn add_disable_pool_auth_test_accs(
    keys: AddDisablePoolAuthIxKeysOwned,
    pool: PoolState,
    disable_pool_auth_list: Vec<[u8; 32]>,
) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewAddDisablePoolAuthIxAccsBuilder::start()
        .with_admin(mock_sys_acc(LAMPORTS))
        .with_payer(mock_sys_acc(LAMPORTS))
        .with_new(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_account(pool))
        .with_disable_pool_auth_list(disable_pool_auth_list_account(disable_pool_auth_list))
        .with_system_program(keyed_account_for_system_program().1)
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `disable_pool_auth_list.last()` at the end of ix
fn add_disable_pool_auth_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> [u8; 32] {
    let (
        _,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec_validate(svm, ix, bef, &[Check::all_rent_exempt()]));
    let aft: AccountMap = resulting_accounts.into_iter().collect();

    assert_balanced(bef, &aft);

    let list_accs = acc_bef_aft(&DISABLE_POOL_AUTHORITY_LIST_ID.into(), bef, &aft);
    let [list_bef, list_aft] =
        list_accs.map(|a| DisablePoolAuthorityList::of_acc_data(&a.data).unwrap().0);
    let list_acc_aft = list_accs[1];

    let new_pk = ix.accounts[ADD_DISABLE_POOL_AUTH_IX_ACCS_IDX_NEW]
        .pubkey
        .to_bytes();

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_diffs_disable_pool_auth_list(
                DisablePoolAuthListChanges::new(list_bef)
                    .with_push(new_pk)
                    .build(),
                list_bef,
                list_aft,
            );
            // at the end of any successful Add, list acc should be owned by prog
            assert_eq!(list_acc_aft.owner, inf1_ctl_jiminy::ID.into());
            assert_valid_disable_pool_auth_list(list_aft);
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }

    new_pk
}

#[test]
fn add_disable_pool_auth_correct_basic() {
    // +69 to avoid using system program [0; 32]
    let [admin, new_auth] = core::array::from_fn(|i| [u8::try_from(i + 69).unwrap(); 32]);
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_admin(admin),
        ..Default::default()
    });
    let keys = NewAddDisablePoolAuthIxAccsBuilder::start()
        .with_admin(admin)
        .with_payer(admin)
        .with_new(new_auth)
        .with_pool_state(POOL_STATE_ID)
        .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
        .with_system_program(SYS_PROG_ID)
        .build();
    let ret = add_disable_pool_auth_test(
        &add_disable_pool_auth_ix(keys),
        &add_disable_pool_auth_test_accs(keys, pool, vec![]),
        Option::<ProgramError>::None,
    );
    assert_eq!(ret, new_auth);
}

fn to_inp(
    (k, ps, list): (AddDisablePoolAuthIxKeysOwned, PoolState, Vec<[u8; 32]>),
) -> (Instruction, AccountMap) {
    (
        add_disable_pool_auth_ix(k),
        add_disable_pool_auth_test_accs(k, ps, list),
    )
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_normal_pk(),
        any_pool_state(Default::default()),
        any_disable_pool_auth_list(0..=MAX_DISABLE_POOL_AUTH_LIST_LEN),
    )
        .prop_map(|(new_auth, payer, ps, list)| {
            (
                NewAddDisablePoolAuthIxAccsBuilder::start()
                    .with_admin(ps.admin)
                    .with_payer(payer)
                    .with_new(new_auth)
                    .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
                    .with_pool_state(POOL_STATE_ID)
                    .with_system_program(SYS_PROG_ID)
                    .build(),
                ps,
                list,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn add_disable_pool_auth_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        add_disable_pool_auth_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(Default::default())
        .prop_flat_map(|ps| {
            (
                any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.admin),
                Just(ps),
                any_normal_pk(),
                any_normal_pk(),
                any_disable_pool_auth_list(0..=MAX_DISABLE_POOL_AUTH_LIST_LEN),
            )
        })
        .prop_map(|(wrong_admin, ps, new_auth, payer, list)| {
            (
                NewAddDisablePoolAuthIxAccsBuilder::start()
                    .with_admin(wrong_admin)
                    .with_payer(payer)
                    .with_new(new_auth)
                    .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
                    .with_pool_state(POOL_STATE_ID)
                    .with_system_program(SYS_PROG_ID)
                    .build(),
                ps,
                list,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn add_disable_pool_auth_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        add_disable_pool_auth_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[ADD_DISABLE_POOL_AUTH_IX_ACCS_IDX_ADMIN].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn add_disable_pool_auth_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        add_disable_pool_auth_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn duplicate_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_disable_pool_auth_list(1..=MAX_DISABLE_POOL_AUTH_LIST_LEN) // must have at least 1 elem for dup
        .prop_flat_map(|list| (0..list.len(), Just(list)))
        .prop_flat_map(|(i, list)| {
            (
                Just(list[i]),
                Just(list),
                any_normal_pk(),
                any_pool_state(Default::default()),
            )
        })
        .prop_map(|(dup, list, payer, ps)| {
            (
                NewAddDisablePoolAuthIxAccsBuilder::start()
                    .with_admin(ps.admin)
                    .with_payer(payer)
                    .with_new(dup)
                    .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
                    .with_pool_state(POOL_STATE_ID)
                    .with_system_program(SYS_PROG_ID)
                    .build(),
                ps,
                list,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn add_disable_pool_auth_duplicate_pt(
        (ix, bef) in duplicate_strat(),
    ) {
        silence_mollusk_logs();
        add_disable_pool_auth_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::DuplicateDisablePoolAuthority))
        );
    }
}

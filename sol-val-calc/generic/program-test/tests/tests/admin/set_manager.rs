use inf1_svc_generic::{
    accounts::state::{State, StatePacked},
    instructions::manager::{
        NewSetManagerIxAccsBuilder, SetManagerIxAccs, SetManagerIxAccsDestr, SetManagerIxData,
        SetManagerIxKeysOwned, SET_MANAGER_IX_ACCS_IDX_CURR, SET_MANAGER_IX_ACCS_IDX_NEW,
        SET_MANAGER_IX_IS_SIGNER, SET_MANAGER_IX_IS_WRITER,
    },
};
use inf1_svc_generic_program_test::{CONST_KEYS_OWNED, CONST_PDAS};
use inf1_test_utils::{
    acc_bef_aft, any_gen_svc_state, any_normal_pk, assert_diffs_gen_svc_state,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_gen_svc_state, mock_sys_acc,
    mollusk_exec, silence_mollusk_logs, AccountMap, Diff, DiffsGenSvcState,
};
use jiminy_entrypoint::program_error::{
    ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE,
};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_manager_ix(keys: SetManagerIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_MANAGER_IX_IS_SIGNER.0.iter(),
        SET_MANAGER_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts,
        data: SetManagerIxData::as_buf().into(),
    }
}

fn set_manager_test_accs(keys: SetManagerIxKeysOwned, state_data: &State) -> AccountMap {
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetManagerIxAccsBuilder::start()
        .with_curr(mock_sys_acc(LAMPORTS))
        .with_new(mock_sys_acc(LAMPORTS))
        .with_state(mock_gen_svc_state(
            *state_data,
            Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        ))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Executes SetManager and returns the resulting manager.
/// If `expected_err` is Some, asserts the error and returns None
fn set_manager_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<[u8; 32]> {
    let result = SVM.with(|svm| mollusk_exec(svm, core::slice::from_ref(ix), bef));

    match expected_err {
        None => {
            let ok = result.unwrap();

            let [state_bef, state_aft] = acc_bef_aft(
                &Pubkey::new_from_array(CONST_PDAS.state().0),
                bef,
                &ok.resulting_accounts,
            )
            .map(|a| StatePacked::of_acc_data(&a.data).unwrap().into_state());

            let expected_new = ix.accounts[SET_MANAGER_IX_ACCS_IDX_NEW].pubkey.to_bytes();
            assert_diffs_gen_svc_state(
                &DiffsGenSvcState {
                    manager: Diff::Changed(state_bef.manager, expected_new),
                    last_upgrade_slot: Diff::Unchanged,
                },
                &state_bef,
                &state_aft,
            );
            Some(state_aft.manager)
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
            None
        }
    }
}

#[test]
fn set_manager_correct_basic() {
    let [curr_admin, new_admin] = core::array::from_fn(|i| [u8::try_from(i).unwrap(); 32]);
    let state = State {
        manager: curr_admin,
        last_upgrade_slot: 0,
    };
    let keys = SetManagerIxAccs::from_destr(SetManagerIxAccsDestr {
        curr: curr_admin,
        new: new_admin,
        state: CONST_PDAS.state().0,
    });
    let ret = set_manager_test(
        &set_manager_ix(keys),
        &set_manager_test_accs(keys, &state),
        Option::<ProgramError>::None,
    )
    .unwrap();
    assert_eq!(ret, new_admin);
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_gen_svc_state()).prop_map(|(new_admin, state)| {
        let keys = SetManagerIxAccs::from_destr(SetManagerIxAccsDestr {
            curr: state.manager,
            new: new_admin,
            state: CONST_PDAS.state().0,
        });
        (set_manager_ix(keys), set_manager_test_accs(keys, &state))
    })
}

proptest! {
    #[test]
    fn set_manager_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        set_manager_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_gen_svc_state())
        .prop_flat_map(|(new_admin, state)| {
            let curr_admin = state.manager;
            (any_normal_pk(), Just(new_admin), Just(state))
                .prop_filter("", move |(wrong_curr, _, _)| *wrong_curr != curr_admin)
        })
        .prop_map(|(wrong_curr, new_admin, state)| {
            let keys = SetManagerIxAccs::from_destr(SetManagerIxAccsDestr {
                curr: wrong_curr,
                new: new_admin,
                state: CONST_PDAS.state().0,
            });
            (set_manager_ix(keys), set_manager_test_accs(keys, &state))
        })
}

proptest! {
    #[test]
    fn set_manager_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_manager_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

fn curr_missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[SET_MANAGER_IX_ACCS_IDX_CURR].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn set_manager_curr_missing_sig_pt(
        (ix, bef) in curr_missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_manager_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

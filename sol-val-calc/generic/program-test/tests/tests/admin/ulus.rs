use inf1_svc_generic::{
    accounts::{
        external::parse_bpf_loader_v3_programdata_meta,
        state::{State, StatePacked},
    },
    instructions::manager::{
        NewULUSIxAccsBuilder, ULUSIxAccs, ULUSIxAccsDestr, ULUSIxData, ULUSIxKeysOwned,
        ULUS_IX_ACCS_IDX_MANAGER, ULUS_IX_IS_SIGNER, ULUS_IX_IS_WRITER,
    },
};
use inf1_svc_generic_program_test::{CONST_KEYS_OWNED, CONST_PDAS};
use inf1_test_utils::{
    acc_bef_aft, any_gen_svc_state, any_normal_pk, assert_diffs_gen_svc_state,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_gen_svc_state, mock_prog_acc,
    mock_progdata_acc, mock_sys_acc, mollusk_exec, silence_mollusk_logs, u64_strat, AccountMap,
    Diff, DiffsGenSvcState, ProgramDataAddr,
};
use jiminy_entrypoint::program_error::{
    ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE,
};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn ulus_ix(keys: ULUSIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        ULUS_IX_IS_SIGNER.0.iter(),
        ULUS_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts,
        data: ULUSIxData::as_buf().into(),
    }
}

fn ulus_test_accs(keys: ULUSIxKeysOwned, state: &State, progdata_slot: u64) -> AccountMap {
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewULUSIxAccsBuilder::start()
        .with_manager(mock_sys_acc(LAMPORTS))
        .with_state(mock_gen_svc_state(
            *state,
            Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        ))
        .with_pool_prog(mock_prog_acc(
            // dontcare, we dont read ProgramData from account data
            ProgramDataAddr::Raw(Default::default()),
        ))
        .with_pool_progdata(mock_progdata_acc(progdata_slot))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Executes ULUS and returns the new last_upgrade_slot on success.
/// If `expected_err` is Some, asserts the error and returns None.
fn ulus_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<u64> {
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

            let progdata_acc = bef
                .get(&Pubkey::new_from_array(CONST_PDAS.pool_progdata().0))
                .unwrap();
            let expected_slot =
                parse_bpf_loader_v3_programdata_meta(progdata_acc.data.first_chunk().unwrap())
                    .unwrap()
                    .0;

            assert_diffs_gen_svc_state(
                &DiffsGenSvcState {
                    manager: Diff::Unchanged,
                    last_upgrade_slot: Diff::Changed(state_bef.last_upgrade_slot, expected_slot),
                },
                &state_bef,
                &state_aft,
            );
            Some(state_aft.last_upgrade_slot)
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
            None
        }
    }
}

#[test]
fn ulus_correct_basic() {
    let manager = [1u8; 32];
    let state = State {
        manager,
        last_upgrade_slot: 0,
    };
    let progdata_slot = 42;
    let keys = ULUSIxAccs::from_destr(ULUSIxAccsDestr {
        manager,
        state: CONST_PDAS.state().0,
        pool_prog: *CONST_KEYS_OWNED.pool_prog(),
        pool_progdata: CONST_PDAS.pool_progdata().0,
    });
    let ret = ulus_test(
        &ulus_ix(keys),
        &ulus_test_accs(keys, &state, progdata_slot),
        Option::<ProgramError>::None,
    )
    .unwrap();
    assert_eq!(ret, progdata_slot);
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_gen_svc_state(), u64_strat(None)).prop_map(|(state, progdata_slot)| {
        let keys = ULUSIxAccs::from_destr(ULUSIxAccsDestr {
            manager: state.manager,
            state: CONST_PDAS.state().0,
            pool_prog: *CONST_KEYS_OWNED.pool_prog(),
            pool_progdata: CONST_PDAS.pool_progdata().0,
        });
        (ulus_ix(keys), ulus_test_accs(keys, &state, progdata_slot))
    })
}

proptest! {
    #[test]
    fn ulus_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        ulus_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_gen_svc_state(), u64_strat(None))
        .prop_flat_map(|(wrong_manager, state, progdata_slot)| {
            (Just(wrong_manager), Just(state), Just(progdata_slot))
                .prop_filter("", move |(wm, s, _)| *wm != s.manager)
        })
        .prop_map(|(wrong_manager, state, progdata_slot)| {
            let keys = ULUSIxAccs::from_destr(ULUSIxAccsDestr {
                manager: wrong_manager,
                state: CONST_PDAS.state().0,
                pool_prog: *CONST_KEYS_OWNED.pool_prog(),
                pool_progdata: CONST_PDAS.pool_progdata().0,
            });
            (ulus_ix(keys), ulus_test_accs(keys, &state, progdata_slot))
        })
}

proptest! {
    #[test]
    fn ulus_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        ulus_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

fn manager_missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[ULUS_IX_ACCS_IDX_MANAGER].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn ulus_manager_missing_sig_pt(
        (ix, bef) in manager_missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        ulus_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

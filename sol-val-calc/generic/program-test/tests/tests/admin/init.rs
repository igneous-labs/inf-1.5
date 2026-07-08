use inf1_svc_generic::{
    accounts::state::{State, StatePacked},
    errs::GenSvcErr,
    instructions::init::{
        InitIxAccs, InitIxData, InitIxKeysOwned, InitIxPreAccs, InitIxPreAccsDestr,
        INIT_IX_IS_SIGNER, INIT_IX_IS_WRITER,
    },
    keys::GLOBAL_CONST_KEYS_OWNED,
};
use inf1_svc_generic_program_test::{CONST_KEYS_OWNED, CONST_PDAS};
use inf1_test_utils::{
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_gen_svc_state, mock_sys_acc,
    mollusk_exec, AccountMap,
};
use jiminy_entrypoint::program_error::ProgramError;
use mollusk_svm::program::keyed_account_for_system_program;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn init_ix(keys: InitIxKeysOwned) -> Instruction {
    let accounts =
        keys_signer_writable_to_metas(keys.seq(), INIT_IX_IS_SIGNER.seq(), INIT_IX_IS_WRITER.seq());
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts,
        data: InitIxData::as_buf().into(),
    }
}

fn init_test(ix: &Instruction, bef: &AccountMap, expected_err: Option<impl Into<ProgramError>>) {
    let result = SVM.with(|svm| mollusk_exec(svm, core::slice::from_ref(ix), bef));

    match expected_err {
        None => {
            let ok = result.unwrap();
            let state_acc = ok
                .resulting_accounts
                .get(&Pubkey::new_from_array(CONST_PDAS.state().0))
                .unwrap();
            let state = StatePacked::of_acc_data(&state_acc.data)
                .unwrap()
                .into_state();
            assert_eq!(state.manager, *CONST_KEYS_OWNED.init_manager());
            assert_eq!(state.last_upgrade_slot, 0);
            assert_eq!(
                state_acc.owner,
                Pubkey::new_from_array(*CONST_KEYS_OWNED.program())
            );
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
        }
    }
}

#[test]
fn init_empty_acc() {
    let payer = [2u8; 32];
    let keys = InitIxKeysOwned {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer,
            state: CONST_PDAS.state().0,
        }),
        sys_prog: *GLOBAL_CONST_KEYS_OWNED.sys_prog(),
    };
    let accs = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: mock_sys_acc(1_000_000_000),
            state: mock_sys_acc(0),
        }),
        sys_prog: keyed_account_for_system_program().1,
    };
    init_test(
        &init_ix(keys),
        &keys
            .seq()
            .copied()
            .map(Into::into)
            .zip(accs.seq().cloned())
            .collect(),
        Option::<ProgramError>::None,
    );
}

#[test]
fn init_nonempty_sys_acc() {
    let payer = [2u8; 32];
    let keys = InitIxKeysOwned {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer,
            state: CONST_PDAS.state().0,
        }),
        sys_prog: *GLOBAL_CONST_KEYS_OWNED.sys_prog(),
    };
    let accs = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: mock_sys_acc(1_000_000_000),
            state: mock_sys_acc(1_000_000_000),
        }),
        sys_prog: keyed_account_for_system_program().1,
    };
    init_test(
        &init_ix(keys),
        &keys
            .seq()
            .copied()
            .map(Into::into)
            .zip(accs.seq().cloned())
            .collect(),
        Option::<ProgramError>::None,
    );
}

#[test]
fn init_already_initialized() {
    let payer = [2u8; 32];
    let keys = InitIxKeysOwned {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer,
            state: CONST_PDAS.state().0,
        }),
        sys_prog: *GLOBAL_CONST_KEYS_OWNED.sys_prog(),
    };
    let accs = InitIxAccs {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            payer: mock_sys_acc(1_000_000_000),
            state: mock_gen_svc_state(
                State {
                    manager: *CONST_KEYS_OWNED.init_manager(),
                    last_upgrade_slot: 0,
                },
                Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
            ),
        }),
        sys_prog: keyed_account_for_system_program().1,
    };
    init_test(
        &init_ix(keys),
        &keys
            .seq()
            .copied()
            .map(Into::into)
            .zip(accs.seq().cloned())
            .collect(),
        Some(ProgramError::custom(
            GenSvcErr::StateAlreadyInitialized.into(),
        )),
    );
}

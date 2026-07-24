use inf1_pp_reserve_v2_core::{
    accounts::{pricing_state_account_size, pricing_state_of_acc_data_packed},
    init::INITIAL_ENTRIES,
    instructions::init::{
        InitIxAccsGen, InitIxData, InitIxPreAccs, InitIxPreAccsDestr, INIT_IX_IS_SIGNER,
        INIT_IX_IS_WRITER,
    },
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
};
use inf1_test_utils::{
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mollusk_exec, AccountMap,
};
use jiminy_cpi::program_error::ILLEGAL_OWNER;
use mollusk_svm::program::keyed_account_for_system_program;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{assert_valid_fee_entries, SVM};

type InitKeysOwned = InitIxAccsGen<[u8; 32]>;

fn init_ix(keys: &InitKeysOwned) -> Instruction {
    let accounts =
        keys_signer_writable_to_metas(keys.seq(), INIT_IX_IS_SIGNER.seq(), INIT_IX_IS_WRITER.seq());
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts,
        data: InitIxData::as_buf().into(),
    }
}

fn init_ix_accounts(keys: &InitKeysOwned) -> AccountMap {
    let InitIxAccsGen { pre, sys_prog } = keys;
    let keys = InitIxPreAccs(pre.0.map(Pubkey::new_from_array));
    let mut am = AccountMap::new();
    am.extend([
        (
            *keys.payer(),
            Account {
                lamports: 10_000_000_000,
                ..Default::default()
            },
        ),
        (
            *keys.pricing_state(),
            Account {
                lamports: 0,
                ..Default::default()
            },
        ),
        (
            Pubkey::new_from_array(*sys_prog),
            keyed_account_for_system_program().1,
        ),
    ]);

    am
}

fn assert_correct_init(resulting_accounts: &AccountMap) {
    let ps_pk = Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state());
    let ps_acc = resulting_accounts.get(&ps_pk).unwrap();

    assert_eq!(
        ps_acc.owner,
        Pubkey::new_from_array(*CONST_KEYS_OWNED.program())
    );
    assert_eq!(ps_acc.data.len(), pricing_state_account_size(2));

    let (admin, entries) = pricing_state_of_acc_data_packed(&ps_acc.data).unwrap();
    let entries: Vec<_> = entries.0.iter().map(|e| e.into_fee_entry()).collect();

    assert_eq!(*admin, *CONST_KEYS_OWNED.init_admin());
    assert_eq!(&entries, &INITIAL_ENTRIES);
    assert_valid_fee_entries(&entries);
}

#[test]
fn init_success_from_empty() {
    let keys = InitIxAccsGen {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
            payer: [2u8; 32],
        }),
        sys_prog: *CONST_KEYS_OWNED.sys_prog(),
    };
    let ix = init_ix(&keys);
    let accs = init_ix_accounts(&keys);
    let ok = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
        .unwrap();
    assert_correct_init(&ok.resulting_accounts);
}

#[test]
fn init_fail_already_initialized() {
    let keys = InitIxAccsGen {
        pre: InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
            payer: [2u8; 32],
        }),
        sys_prog: *CONST_KEYS_OWNED.sys_prog(),
    };
    let ix = init_ix(&keys);
    let mut accs = init_ix_accounts(&keys);

    let ps_pk = Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state());
    accs.insert(
        ps_pk,
        Account {
            lamports: 1_000_000_000,
            owner: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
            data: vec![0u8; pricing_state_account_size(2)],
            ..Default::default()
        },
    );

    let err = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
        .unwrap_err();
    assert_jiminy_prog_err(&err, ILLEGAL_OWNER);
}

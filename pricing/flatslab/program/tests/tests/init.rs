use inf1_pp_flatslab_core::{
    accounts::Slab,
    instructions::init::{
        InitIxAccs, InitIxData, InitIxKeysOwned, NewInitIxAccsBuilder, INIT_IX_IS_SIGNER,
        INIT_IX_IS_WRITER,
    },
    keys::{INITIAL_ADMIN_ID, LP_MINT_ID, SLAB_ID},
    typedefs::SlabEntryPacked,
    ID,
};
use inf1_pp_flatslab_program::SYS_PROG_ID;
use jiminy_cpi::program_error::INVALID_ACCOUNT_DATA;
use mollusk_svm::{
    program::keyed_account_for_system_program,
    result::{Check, InstructionResult, ProgramResult},
};
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{
    mollusk::{silence_mollusk_logs, MOLLUSK},
    props::{non_slab_pks, slab_for_swap, MAX_MINTS},
    solana::{keys_signer_writable_to_metas, PkAccountTup},
    tests::should_fail_with_program_err,
};

fn init_ix(keys: &InitIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        INIT_IX_IS_SIGNER.0.iter(),
        INIT_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: InitIxData::new().as_buf().into(),
    }
}

fn init_ix_accounts(
    keys: &InitIxKeysOwned,
    payer_lamports: u64,
    slab_lamports: u64,
) -> InitIxAccs<PkAccountTup> {
    NewInitIxAccsBuilder::start()
        .with_payer((
            Pubkey::new_from_array(*keys.payer()),
            Account {
                lamports: payer_lamports,
                ..Default::default()
            },
        ))
        .with_slab((
            Pubkey::new_from_array(*keys.slab()),
            Account {
                lamports: slab_lamports,
                ..Default::default()
            },
        ))
        .with_system_program((
            Pubkey::new_from_array(*keys.system_program()),
            keyed_account_for_system_program().1,
        ))
        .build()
}

fn assert_correct_init(resulting_accounts: &[(Pubkey, Account)]) {
    let (_, slab) = resulting_accounts
        .iter()
        .find(|(pk, _)| *pk.as_array() == SLAB_ID)
        .unwrap();
    let slab = Slab::of_acc_data(&slab.data).unwrap();
    assert_eq!(*slab.admin(), INITIAL_ADMIN_ID);
    assert_eq!(slab.entries().0.len(), 1);
    assert_eq!(
        *slab.entries().find_by_mint(&LP_MINT_ID).unwrap(),
        SlabEntryPacked::INITIAL_LP
    );
}

/// Enought to pay for `INIT_SLAB_RENT_EXEMPT_LAMPORTS` without itself becoming not rent-exempt
const PAYER_MIN_LAMPORTS: u64 = 2_500_000;

const INIT_SLAB_RENT_EXEMPT_LAMPORTS: u64 = 1_392_000;

proptest! {
    #[test]
    fn init_success(
        payer_pk in non_slab_pks().prop_filter("Must not be sys prog", |v| *v != SYS_PROG_ID),
        payer_lamports in PAYER_MIN_LAMPORTS..=u64::MAX,
        slab_lamports in 0..=u64::MAX - INIT_SLAB_RENT_EXEMPT_LAMPORTS, // avoid overflow
    ) {
        silence_mollusk_logs();

        let keys = NewInitIxAccsBuilder::start()
            .with_system_program(SYS_PROG_ID)
            .with_payer(payer_pk)
            .with_slab(SLAB_ID)
            .build();
        let ix = init_ix(&keys);
        let accs = init_ix_accounts(&keys, payer_lamports, slab_lamports);
        MOLLUSK.with(|mollusk| {
            let InstructionResult {
                program_result,
                resulting_accounts,
                ..
            } = mollusk.process_and_validate_instruction(
                &ix,
                &accs.0,
                &[Check::all_rent_exempt()],
            );
            assert_eq!(program_result, ProgramResult::Success);
            assert_correct_init(&resulting_accounts);
        });
    }
}

proptest! {
    #[test]
    fn init_fails_if_already_init(
        (slab_data, _, _) in slab_for_swap(MAX_MINTS),
        payer_pk in non_slab_pks().prop_filter("Must not be sys prog", |v| *v != SYS_PROG_ID),
        payer_lamports in PAYER_MIN_LAMPORTS..=u64::MAX,
        slab_lamports in INIT_SLAB_RENT_EXEMPT_LAMPORTS..=u64::MAX,
    ) {
        silence_mollusk_logs();

        let keys = NewInitIxAccsBuilder::start()
            .with_system_program(SYS_PROG_ID)
            .with_payer(payer_pk)
            .with_slab(SLAB_ID)
            .build();
        let ix = init_ix(&keys);
        let mut accs = init_ix_accounts(&keys, payer_lamports, slab_lamports);
        accs.slab_mut().1 = Account {
            lamports: slab_lamports,
            owner: ID.into(),
            data: slab_data,
            ..Default::default()
        };

        should_fail_with_program_err(
            &ix,
            &accs.0,
            INVALID_ACCOUNT_DATA,
        );
    }
}

proptest! {
    #[test]
    fn init_fails_if_owner_wrong(
        invalid_owner in non_slab_pks().prop_filter("Must not be sys prog", |v| *v != SYS_PROG_ID),
        payer_pk in non_slab_pks().prop_filter("Must not be sys prog", |v| *v != SYS_PROG_ID),
        payer_lamports in PAYER_MIN_LAMPORTS..=u64::MAX,
        slab_lamports in 0..=u64::MAX - INIT_SLAB_RENT_EXEMPT_LAMPORTS, // avoid overflow
    ) {
        silence_mollusk_logs();

        let keys = NewInitIxAccsBuilder::start()
            .with_system_program(SYS_PROG_ID)
            .with_payer(payer_pk)
            .with_slab(SLAB_ID)
            .build();
        let ix = init_ix(&keys);
        let mut accs = init_ix_accounts(&keys, payer_lamports, slab_lamports);
        accs.slab_mut().1 = Account {
            lamports: slab_lamports,
            owner: invalid_owner.into(),
            ..Default::default()
        };

        should_fail_with_program_err(
            &ix,
            &accs.0,
            INVALID_ACCOUNT_DATA,
        );
    }
}

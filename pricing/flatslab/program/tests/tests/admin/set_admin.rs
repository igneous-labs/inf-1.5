use inf1_pp_flatslab_core::{
    accounts::Slab,
    errs::FlatSlabProgramErr,
    instructions::admin::set_admin::{
        NewSetAdminIxAccsBuilder, SetAdminIxAccs, SetAdminIxData, SetAdminIxKeysOwned,
        SET_ADMIN_IX_ACCS_IDX_CURRENT_ADMIN, SET_ADMIN_IX_IS_SIGNER, SET_ADMIN_IX_IS_WRITER,
    },
    keys::SLAB_ID,
    ID,
};
use inf1_test_utils::{keys_signer_writable_to_metas, silence_mollusk_logs, PkAccountTup};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{
    mollusk::SVM,
    props::{slab_data, MAX_MINTS},
    solana::slab_account,
    tests::should_fail_with_flatslab_prog_err,
};

fn set_admin_ix(keys: &SetAdminIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_ADMIN_IX_IS_SIGNER.0.iter(),
        SET_ADMIN_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetAdminIxData::new().as_buf().into(),
    }
}

fn set_admin_ix_accounts(
    keys: &SetAdminIxKeysOwned,
    slab_data: Vec<u8>,
) -> SetAdminIxAccs<PkAccountTup> {
    NewSetAdminIxAccsBuilder::start()
        .with_slab((
            Pubkey::new_from_array(*keys.slab()),
            slab_account(slab_data),
        ))
        .with_current_admin((
            Pubkey::new_from_array(*keys.current_admin()),
            Default::default(),
        ))
        .with_new_admin((
            Pubkey::new_from_array(*keys.new_admin()),
            Default::default(),
        ))
        .build()
}

fn assert_admin(resulting_accounts: &[PkAccountTup], expected_admin: &[u8; 32]) {
    let (_, slab) = resulting_accounts
        .iter()
        .find(|(pk, _)| *pk.as_array() == SLAB_ID)
        .unwrap();
    let slab = Slab::of_acc_data(&slab.data).unwrap();
    assert_eq!(slab.admin(), expected_admin);
}

proptest! {
    #[test]
    fn set_admin_success(
        slab in slab_data(0..=MAX_MINTS),
        new_admin: [u8; 32],
    ) {
        silence_mollusk_logs();

        let current_admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewSetAdminIxAccsBuilder::start()
            .with_current_admin(current_admin)
            .with_new_admin(new_admin)
            .with_slab(SLAB_ID)
            .build();
        let ix = set_admin_ix(&keys);
        let accs = set_admin_ix_accounts(&keys, slab);
        SVM.with(|mollusk| {
            let InstructionResult {
                program_result,
                resulting_accounts,
                ..
            } = mollusk.process_instruction(
                &ix,
                &accs.0,
            );
            assert_eq!(program_result, ProgramResult::Success);
            assert_admin(&resulting_accounts, &new_admin);
        });
    }
}

proptest! {
    #[test]
    fn set_admin_fails_if_no_sig(
        slab in slab_data(0..=MAX_MINTS),
        new_admin: [u8; 32],
    ) {
        silence_mollusk_logs();

        let current_admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewSetAdminIxAccsBuilder::start()
            .with_current_admin(current_admin)
            .with_new_admin(new_admin)
            .with_slab(SLAB_ID)
            .build();
        let mut ix = set_admin_ix(&keys);
        ix.accounts[SET_ADMIN_IX_ACCS_IDX_CURRENT_ADMIN].is_signer = false;
        let accs = set_admin_ix_accounts(&keys, slab);
        should_fail_with_flatslab_prog_err(&ix, &accs.0, FlatSlabProgramErr::MissingAdminSignature);
    }
}

proptest! {
    #[test]
    fn set_admin_fails_if_wrong_current_admin(
        slab in slab_data(0..=MAX_MINTS),
        wrong_current_admin: [u8; 32],
        new_admin: [u8; 32],
    ) {
        let current_admin = *Slab::of_acc_data(&slab).unwrap().admin();
        if wrong_current_admin == current_admin {
            return Ok(());
        }

        silence_mollusk_logs();

        let keys = NewSetAdminIxAccsBuilder::start()
            .with_current_admin(wrong_current_admin)
            .with_new_admin(new_admin)
            .with_slab(SLAB_ID)
            .build();
        let ix = set_admin_ix(&keys);
        let accs = set_admin_ix_accounts(&keys, slab);
        should_fail_with_flatslab_prog_err(&ix, &accs.0, FlatSlabProgramErr::MissingAdminSignature);
    }
}

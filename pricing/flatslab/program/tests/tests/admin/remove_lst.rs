use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    accounts::Slab,
    errs::FlatSlabProgramErr,
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxAccs, RemoveLstIxData, RemoveLstIxKeysOwned,
        REMOVE_LST_IX_ACCS_IDX_ADMIN, REMOVE_LST_IX_IS_SIGNER, REMOVE_LST_IX_IS_WRITER,
    },
    keys::SLAB_ID,
    ID,
};
use mollusk_svm::result::{Check, InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::{
        mollusk::{silence_mollusk_logs, MOLLUSK},
        props::{rand_unknown_pk, slab_data, slab_for_swap, MAX_MINTS},
        solana::{keys_signer_writable_to_metas, slab_account, PkAccountTup},
        tests::should_fail_with_flatslab_prog_err,
    },
    tests::admin::{assert_slab_entry_on_slab, assert_valid_slab},
};

fn remove_lst_ix(keys: &RemoveLstIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        REMOVE_LST_IX_IS_SIGNER.0.iter(),
        REMOVE_LST_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: RemoveLstIxData::new().as_buf().into(),
    }
}

fn remove_lst_ix_accounts(
    keys: &RemoveLstIxKeysOwned,
    slab_data: Vec<u8>,
) -> RemoveLstIxAccs<PkAccountTup> {
    NewRemoveLstIxAccsBuilder::start()
        .with_slab((
            Pubkey::new_from_array(*keys.slab()),
            slab_account(slab_data),
        ))
        .with_admin((Pubkey::new_from_array(*keys.admin()), Default::default()))
        .with_mint((Pubkey::new_from_array(*keys.mint()), Default::default()))
        .with_refund_rent_to((
            Pubkey::new_from_array(*keys.refund_rent_to()),
            Account {
                lamports: 890_880, // solana rent 0
                ..Default::default()
            },
        ))
        .build()
}

fn assert_slab_entry_removed(slab_acc_data: &[u8], mint: &[u8; 32]) {
    let slab_entries = Slab::of_acc_data(slab_acc_data).unwrap().entries();
    assert!(slab_entries.find_by_mint(mint).is_err());
}

fn assert_old_slab_entries_untouched(old_slab_data: &[u8], new_slab_data: &[u8], excl: &[u8; 32]) {
    let old = Slab::of_acc_data(old_slab_data).unwrap().entries();
    for old_e in old.0 {
        if old_e.mint() != excl {
            assert_slab_entry_on_slab(new_slab_data, old_e);
        }
    }
}

fn remove_lst_success_test(
    removed_mint: &[u8; 32],
    old_slab: &[u8],
    ix: &Instruction,
    accs: RemoveLstIxAccs<PkAccountTup>,
) {
    MOLLUSK.with(|mollusk| {
        let InstructionResult {
            program_result,
            resulting_accounts,
            ..
        } = mollusk.process_and_validate_instruction(ix, &accs.0, &[Check::all_rent_exempt()]);
        assert_eq!(program_result, ProgramResult::Success);
        let (_, new_slab) = resulting_accounts
            .iter()
            .find(|(pk, _)| *pk.as_array() == SLAB_ID)
            .unwrap();
        assert_valid_slab(&new_slab.data);
        assert_slab_entry_removed(&new_slab.data, removed_mint);
        assert_old_slab_entries_untouched(old_slab, &new_slab.data, removed_mint);
    });
}

proptest! {
    #[test]
    fn remove_lst_success_rand_mint(
        slab in slab_data(0..=MAX_MINTS),
        rrt in rand_unknown_pk(),
        mint in rand_unknown_pk(),
    ) {
        silence_mollusk_logs();

        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewRemoveLstIxAccsBuilder::start()
            .with_admin(admin)
            .with_mint(mint)
            .with_refund_rent_to(rrt)
            .with_slab(SLAB_ID)
            .build();
        let ix = remove_lst_ix(&keys);
        let accs = remove_lst_ix_accounts(&keys, slab.clone());
        remove_lst_success_test(&mint, &slab, &ix, accs);
    }
}

proptest! {
    #[test]
    fn remove_lst_success_mint_on_slab(
        (slab, Pair { inp: mint, .. }, _) in slab_for_swap(MAX_MINTS),
        rrt in rand_unknown_pk(),
    ) {
        silence_mollusk_logs();

        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewRemoveLstIxAccsBuilder::start()
            .with_admin(admin)
            .with_mint(mint)
            .with_refund_rent_to(rrt)
            .with_slab(SLAB_ID)
            .build();
        let ix = remove_lst_ix(&keys);
        let accs = remove_lst_ix_accounts(&keys, slab.clone());
        remove_lst_success_test(&mint, &slab, &ix, accs);
    }
}

proptest! {
    #[test]
    fn remove_lst_fails_if_no_sig(
        slab in slab_data(0..=MAX_MINTS),
        rrt in rand_unknown_pk(),
        mint in rand_unknown_pk(),
    ) {
        silence_mollusk_logs();

        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewRemoveLstIxAccsBuilder::start()
            .with_admin(admin)
            .with_mint(mint)
            .with_refund_rent_to(rrt)
            .with_slab(SLAB_ID)
            .build();
        let mut ix = remove_lst_ix(&keys);
        ix.accounts[REMOVE_LST_IX_ACCS_IDX_ADMIN].is_signer = false;
        let accs = remove_lst_ix_accounts(&keys, slab);
        should_fail_with_flatslab_prog_err(&ix, &accs.0, FlatSlabProgramErr::MissingAdminSignature);
    }
}

proptest! {
    #[test]
    fn remove_lst_fails_if_wrong_admin(
        slab in slab_data(0..=MAX_MINTS),
        wrong_admin: [u8; 32],
        rrt in rand_unknown_pk(),
        mint in rand_unknown_pk(),
    ) {
        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        if wrong_admin == admin {
            return Ok(());
        }

        silence_mollusk_logs();

        let keys = NewRemoveLstIxAccsBuilder::start()
            .with_admin(wrong_admin)
            .with_mint(mint)
            .with_refund_rent_to(rrt)
            .with_slab(SLAB_ID)
            .build();
        let ix = remove_lst_ix(&keys);
        let accs = remove_lst_ix_accounts(&keys, slab);
        should_fail_with_flatslab_prog_err(&ix, &accs.0, FlatSlabProgramErr::MissingAdminSignature);
    }
}

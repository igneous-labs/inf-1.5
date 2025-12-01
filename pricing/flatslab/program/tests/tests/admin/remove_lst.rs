use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    accounts::{Slab, SlabMut},
    errs::FlatSlabProgramErr,
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxData, RemoveLstIxKeysOwned,
        REMOVE_LST_IX_ACCS_IDX_ADMIN, REMOVE_LST_IX_IS_SIGNER, REMOVE_LST_IX_IS_WRITER,
    },
    keys::{LP_MINT_ID, SLAB_ID},
    ID,
};
use inf1_test_utils::{
    keys_signer_writable_to_metas, mollusk_exec, silence_mollusk_logs, AccountMap,
};
use mollusk_svm::result::InstructionResult;
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::{
        accounts::slab_account,
        mollusk::SVM,
        props::{clean_valid_slab, rand_unknown_pk, slab_data, slab_for_swap, MAX_MINTS},
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

fn remove_lst_ix_accounts(keys: &RemoveLstIxKeysOwned, slab_data: Vec<u8>) -> AccountMap {
    let accs = NewRemoveLstIxAccsBuilder::start()
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
        .build();
    accs.0.into_iter().collect()
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
    ix: Instruction,
    accs: AccountMap,
) {
    SVM.with(|mollusk| {
        let aft = mollusk_exec(mollusk, &[ix], &accs)
            .unwrap()
            .resulting_accounts;
        let (_, new_slab) = aft
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
        remove_lst_success_test(&mint, &slab, ix, accs);
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
        remove_lst_success_test(&mint, &slab, ix, accs);
    }
}

proptest! {
    #[test]
    fn remove_lst_fails_if_lp_mint(
        slab in slab_data(0..=MAX_MINTS),
        rrt in rand_unknown_pk(),
    ) {
        silence_mollusk_logs();

        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewRemoveLstIxAccsBuilder::start()
            .with_admin(admin)
            .with_mint(LP_MINT_ID)
            .with_refund_rent_to(rrt)
            .with_slab(SLAB_ID)
            .build();
        let ix = remove_lst_ix(&keys);
        let accs = remove_lst_ix_accounts(&keys, slab.clone());
        should_fail_with_flatslab_prog_err(ix, &accs, FlatSlabProgramErr::CantRemoveLpMint);
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
        should_fail_with_flatslab_prog_err(ix, &accs, FlatSlabProgramErr::MissingAdminSignature);
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
        should_fail_with_flatslab_prog_err(ix, &accs, FlatSlabProgramErr::MissingAdminSignature);
    }
}

/// Check that RemoveLst dont take up way too many CUs
/// (otherwise we might brick the acc if >1.4M CUs to edit account)
#[test]
fn remove_lst_cu_upper_limit() {
    const CU_UPPER_LIMIT: u64 = 50_000;
    const N_ENTRIES: usize = 100_000;
    const SMALLEST_MINT: [u8; 32] = {
        // cannot use SYS_PROG_ID directly
        // or will have issues with duplicate mollusk accounts
        let mut res = [0u8; 32];
        res[31] = 1;
        res
    };

    silence_mollusk_logs();

    let mut rng = rand::rng();
    let mut bytes = vec![0u8; Slab::account_size(N_ENTRIES)];
    rng.fill_bytes(&mut bytes);
    let mut slab_data = clean_valid_slab(bytes);

    // worst-case: removing entry from start of array means
    // having to shift entire array left
    let mut slab = SlabMut::of_acc_data(&mut slab_data).unwrap();
    let entries = slab.as_mut().1 .0;
    // sys prog is the only pk smaller than SMALLEST_MINT
    if *entries[0].mint() == [0u8; 32] {
        return;
    }
    *entries[0].mint_mut() = SMALLEST_MINT;

    let admin = *slab.as_slab().admin();
    let keys = NewRemoveLstIxAccsBuilder::start()
        .with_admin(admin)
        .with_mint(SMALLEST_MINT)
        .with_refund_rent_to(Pubkey::new_unique().to_bytes())
        .with_slab(SLAB_ID)
        .build();
    let ix = remove_lst_ix(&keys);
    let accs = remove_lst_ix_accounts(&keys, slab_data);

    SVM.with(|mollusk| {
        let mut accs_vec: Vec<_> = accs.iter().map(|(k, v)| (*k, v.clone())).collect();
        accs_vec.sort_by_key(|(k, _)| *k);
        let InstructionResult {
            compute_units_consumed,
            ..
        } = mollusk.process_instruction(&ix, &accs_vec);
        assert!(
            compute_units_consumed < CU_UPPER_LIMIT,
            "{compute_units_consumed}"
        );
    });
}

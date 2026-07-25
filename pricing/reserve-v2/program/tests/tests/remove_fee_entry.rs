use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_of_acc_data_packed,
    errs::ReserveV2ProgramErr,
    instructions::admin::{
        common::{AdminIxPreAccs, AdminIxPreAccsDestr, ADMIN_IX_PRE_ACCS_IDX_ADMIN},
        remove_fee_entry::{
            RemoveFeeEntryIxAccsGen, RemoveFeeEntryIxData, RemoveFeeEntryIxSufAccs,
            RemoveFeeEntryIxSufAccsDestr, REMOVE_FEE_ENTRY_IX_IS_SIGNER,
            REMOVE_FEE_ENTRY_IX_IS_WRITER,
        },
    },
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
};
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_reserve_v2_pricing_state, assert_diffs_pricing_state,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_reserve_v2_pricing_state_account,
    mollusk_exec, silence_mollusk_logs, AccountMap, Diff, ListChanges,
};
use jiminy_cpi::program_error::{INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use jiminy_sysvar_rent::Rent;
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{assert_valid_fee_entries, SVM};

type RemoveFeeEntryKeysOwned = RemoveFeeEntryIxAccsGen<[u8; 32]>;

fn remove_fee_entry_ix(keys: &RemoveFeeEntryKeysOwned) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(
            keys.seq(),
            REMOVE_FEE_ENTRY_IX_IS_SIGNER.seq(),
            REMOVE_FEE_ENTRY_IX_IS_WRITER.seq(),
        ),
        data: RemoveFeeEntryIxData::as_buf().into(),
    }
}

fn remove_fee_entry_accs(keys: &RemoveFeeEntryKeysOwned, ps: Account) -> AccountMap {
    AccountMap::from([
        (
            Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state()),
            ps,
        ),
        (
            Pubkey::new_from_array(*keys.pre.admin()),
            Account::default(),
        ),
        (Pubkey::new_from_array(*keys.suf.mint()), Account::default()),
        (
            Pubkey::new_from_array(*keys.suf.refund_rent_to()),
            Account {
                lamports: Rent::default().min_balance(0),
                ..Default::default()
            },
        ),
    ])
}

fn remove_fee_entry_test(keys: &RemoveFeeEntryKeysOwned, ps_bef: Account) {
    let ps_pk = Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state());
    let ix = remove_fee_entry_ix(keys);
    let accs = remove_fee_entry_accs(keys, ps_bef.clone());
    let ok = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
        .unwrap();

    let bef_aft_ps_acc = acc_bef_aft(&ps_pk, &accs, &ok.resulting_accounts);
    let [(bef_admin, bef_entries), (aft_admin, aft_entries)] = bef_aft_ps_acc.map(|a| {
        let (admin, entries_packed) = pricing_state_of_acc_data_packed(&a.data).unwrap();
        (
            admin,
            entries_packed
                .0
                .iter()
                .map(|e| e.into_fee_entry())
                .collect::<Vec<_>>(),
        )
    });

    let mint = *keys.suf.mint();
    let changes = match bef_entries.iter().position(|e| e.mint == mint) {
        Some(idx) => ListChanges::new(&bef_entries).with_del(idx).build(),
        None => ListChanges::new(&bef_entries).build(),
    };

    assert_diffs_pricing_state(
        (Diff::Unchanged, changes),
        (bef_admin, &bef_entries),
        (aft_admin, &aft_entries),
    );
    assert_valid_fee_entries(&aft_entries);
}

fn remove_fee_entry_err_test(
    keys: &RemoveFeeEntryKeysOwned,
    ps: Account,
    expected_err: impl Into<jiminy_entrypoint::program_error::ProgramError>,
) {
    let ix = remove_fee_entry_ix(keys);
    let accs = remove_fee_entry_accs(keys, ps);
    let err = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
        .unwrap_err();
    assert_jiminy_prog_err(&err, expected_err);
}

fn remove_fee_entry_success_strat() -> impl Strategy<Value = (RemoveFeeEntryKeysOwned, Account)> {
    (any_reserve_v2_pricing_state(1usize..4), any_normal_pk())
        .prop_flat_map(|((admin, entries), refund_rent_to)| {
            let extra: Vec<usize> = entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.mint != *CONST_KEYS_OWNED.lp_mint() && e.mint != *CONST_KEYS_OWNED.wsol_mint()
                })
                .map(|(i, _)| i)
                .collect();
            (
                Just(admin),
                Just(entries),
                0..extra.len(),
                Just(refund_rent_to),
            )
        })
        .prop_map(|(admin, entries, rand_extra_idx, refund_rent_to)| {
            let extra: Vec<usize> = entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.mint != *CONST_KEYS_OWNED.lp_mint() && e.mint != *CONST_KEYS_OWNED.wsol_mint()
                })
                .map(|(i, _)| i)
                .collect();
            let mint = entries[extra[rand_extra_idx]].mint;
            let ps = mock_reserve_v2_pricing_state_account(admin, &entries);
            let keys = RemoveFeeEntryIxAccsGen {
                pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                    pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                    admin,
                }),
                suf: RemoveFeeEntryIxSufAccs::from_destr(RemoveFeeEntryIxSufAccsDestr {
                    mint,
                    refund_rent_to,
                }),
            };
            (keys, ps)
        })
}

proptest! {
    #[test]
    fn remove_fee_entry_success((keys, ps) in remove_fee_entry_success_strat()) {
        silence_mollusk_logs();
        remove_fee_entry_test(&keys, ps);
    }
}

fn remove_fee_entry_idempotent_strat() -> impl Strategy<Value = (RemoveFeeEntryKeysOwned, Account)>
{
    (any_reserve_v2_pricing_state(0usize..4), any_normal_pk())
        .prop_flat_map(|((admin, entries), refund_rent_to)| {
            let entries_clone = entries.clone();
            let missing = any_normal_pk().prop_filter("not in entries", move |pk| {
                !entries_clone.iter().any(|e| e.mint == *pk)
            });
            (Just(admin), Just(entries), missing, Just(refund_rent_to))
        })
        .prop_map(|(admin, entries, missing_mint, refund_rent_to)| {
            let ps = mock_reserve_v2_pricing_state_account(admin, &entries);
            let keys = RemoveFeeEntryIxAccsGen {
                pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                    pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                    admin,
                }),
                suf: RemoveFeeEntryIxSufAccs::from_destr(RemoveFeeEntryIxSufAccsDestr {
                    mint: missing_mint,
                    refund_rent_to,
                }),
            };
            (keys, ps)
        })
}

proptest! {
    #[test]
    fn remove_fee_entry_idempotent((keys, ps) in remove_fee_entry_idempotent_strat()) {
        silence_mollusk_logs();
        remove_fee_entry_test(&keys, ps);
    }
}

fn remove_fee_entry_reject_required_mint_strat(
    required_mint: [u8; 32],
) -> impl Strategy<Value = (RemoveFeeEntryKeysOwned, Account)> {
    (any_reserve_v2_pricing_state(0usize..4), any_normal_pk()).prop_map(
        move |((admin, entries), refund_rent_to)| {
            let ps = mock_reserve_v2_pricing_state_account(admin, &entries);
            let keys = RemoveFeeEntryIxAccsGen {
                pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                    pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                    admin,
                }),
                suf: RemoveFeeEntryIxSufAccs::from_destr(RemoveFeeEntryIxSufAccsDestr {
                    mint: required_mint,
                    refund_rent_to,
                }),
            };
            (keys, ps)
        },
    )
}

proptest! {
    #[test]
    fn remove_fee_entry_rejects_lp_mint(
        (keys, ps) in remove_fee_entry_reject_required_mint_strat(*CONST_KEYS_OWNED.lp_mint())
    ) {
        silence_mollusk_logs();
        remove_fee_entry_err_test(
            &keys,
            ps,
            CustomProgErr(ReserveV2ProgramErr::CantRemoveRequiredMint)
        );
    }
}

proptest! {
    #[test]
    fn remove_fee_entry_rejects_wsol_mint(
        (keys, ps) in remove_fee_entry_reject_required_mint_strat(*CONST_KEYS_OWNED.wsol_mint())
    ) {
        silence_mollusk_logs();
        remove_fee_entry_err_test(
            &keys,
            ps,
            CustomProgErr(ReserveV2ProgramErr::CantRemoveRequiredMint)
        );
    }
}

fn remove_fee_entry_wrong_admin_strat() -> impl Strategy<Value = (RemoveFeeEntryKeysOwned, Account)>
{
    (any_reserve_v2_pricing_state(0usize..4), any_normal_pk())
        .prop_flat_map(|((admin, entries), refund_rent_to)| {
            let wrong = any_normal_pk().prop_filter("differs from stored", move |pk| *pk != admin);
            (Just(admin), Just(entries), wrong, Just(refund_rent_to))
        })
        .prop_map(|(admin, entries, wrong_admin, refund_rent_to)| {
            let ps = mock_reserve_v2_pricing_state_account(admin, &entries);
            let keys = RemoveFeeEntryIxAccsGen {
                pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                    pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                    admin: wrong_admin,
                }),
                suf: RemoveFeeEntryIxSufAccs::from_destr(RemoveFeeEntryIxSufAccsDestr {
                    mint: entries[0].mint,
                    refund_rent_to,
                }),
            };
            (keys, ps)
        })
}

proptest! {
    #[test]
    fn remove_fee_entry_wrong_admin((keys, ps) in remove_fee_entry_wrong_admin_strat()) {
        silence_mollusk_logs();
        remove_fee_entry_err_test(&keys, ps, INVALID_ARGUMENT);
    }
}

proptest! {
    #[test]
    fn remove_fee_entry_missing_sig((keys, ps) in remove_fee_entry_success_strat()) {
        silence_mollusk_logs();
        let mut ix = remove_fee_entry_ix(&keys);
        ix.accounts[ADMIN_IX_PRE_ACCS_IDX_ADMIN].is_signer = false;
        let accs = remove_fee_entry_accs(&keys, ps);
        let err = SVM
            .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
            .unwrap_err();
        assert_jiminy_prog_err(&err, MISSING_REQUIRED_SIGNATURE);
    }
}

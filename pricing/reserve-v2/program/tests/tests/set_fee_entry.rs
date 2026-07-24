use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_of_acc_data_packed,
    instructions::admin::{
        common::{AdminIxPreAccs, AdminIxPreAccsDestr, ADMIN_IX_PRE_ACCS_IDX_ADMIN},
        set_fee_entry::{
            SetFeeEntryIxAccsGen, SetFeeEntryIxData, SetFeeEntryIxSufAccs,
            SetFeeEntryIxSufAccsDestr, SET_FEE_ENTRY_IX_IS_SIGNER, SET_FEE_ENTRY_IX_IS_WRITER,
        },
    },
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
    typedefs::{
        FeeEntry, FeeEntryGen, FeeEntryNanos, FeeEntryNanosDestr, FeeNanos, ThresholdNanos,
    },
};
use inf1_test_utils::{
    any_normal_pk, any_reserve_v2_pricing_state, assert_diffs_pricing_state,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_reserve_v2_pricing_state_account,
    mollusk_exec, silence_mollusk_logs, AccountMap, Diff, ListChange, ListChanges,
};
use jiminy_cpi::program_error::{INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::program::keyed_account_for_system_program;
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{assert_valid_fee_entries, SVM};

type SetFeeEntryKeysOwned = SetFeeEntryIxAccsGen<[u8; 32]>;

fn set_fee_entry_ix(keys: &SetFeeEntryKeysOwned, data: &SetFeeEntryIxData) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(
            keys.seq(),
            SET_FEE_ENTRY_IX_IS_SIGNER.seq(),
            SET_FEE_ENTRY_IX_IS_WRITER.seq(),
        ),
        data: data.as_buf().to_vec(),
    }
}

fn set_fee_entry_accs(keys: &SetFeeEntryKeysOwned, ps: Account) -> AccountMap {
    let mut am = AccountMap::new();
    am.extend(
        [
            (
                Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state()),
                ps,
            ),
            keyed_account_for_system_program(),
            (
                Pubkey::new_from_array(*keys.suf.payer()),
                Account {
                    // enough to pay for rent for any len
                    lamports: 1_000_000_000_000,
                    ..Default::default()
                },
            ),
        ]
        .into_iter()
        .chain(
            [keys.pre.admin(), keys.suf.mint()]
                .map(|a| (Pubkey::new_from_array(*a), Default::default())),
        ),
    );
    am
}

fn set_fee_entry_test(keys: &SetFeeEntryKeysOwned, ps_bef: Account, data: &SetFeeEntryIxData) {
    let ps_pk = Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state());
    let ix = set_fee_entry_ix(keys, data);
    let accs = set_fee_entry_accs(keys, ps_bef.clone());
    let ok = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
        .unwrap();

    let (bef_admin, bef_entries) = {
        let bef = &accs[&ps_pk];
        let (admin, entries_packed) = pricing_state_of_acc_data_packed(&bef.data).unwrap();
        (
            admin,
            entries_packed
                .0
                .iter()
                .map(|e| e.into_fee_entry())
                .collect::<Vec<_>>(),
        )
    };
    let (aft_admin, aft_entries) = {
        let aft = &ok.resulting_accounts[&ps_pk];
        let (admin, entries_packed) = pricing_state_of_acc_data_packed(&aft.data).unwrap();
        (
            admin,
            entries_packed
                .0
                .iter()
                .map(|e| e.into_fee_entry())
                .collect::<Vec<_>>(),
        )
    };

    let mint = *keys.suf.mint();
    let (t, ref fees) =
        SetFeeEntryIxData::parse_no_discm(data.as_buf().last_chunk().unwrap()).unwrap();
    let new_threshold = t.get();
    let new_fees = [
        fees.base_fee().get(),
        fees.threshold_fee().get(),
        fees.max_fee().get(),
        fees.output_fee().get(),
    ];

    let changes = match bef_entries.iter().position(|e| e.mint == mint) {
        Some(idx) => {
            let fee = &bef_entries[idx].fee_nanos;
            let diff = FeeEntryGen {
                mint: Diff::Unchanged,
                threshold_nanos: Diff::StrictChanged(
                    bef_entries[idx].threshold_nanos,
                    new_threshold,
                ),
                fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
                    base_fee: Diff::StrictChanged(fee.0[0], new_fees[0]),
                    threshold_fee: Diff::StrictChanged(fee.0[1], new_fees[1]),
                    max_fee: Diff::StrictChanged(fee.0[2], new_fees[2]),
                    output_fee: Diff::StrictChanged(fee.0[3], new_fees[3]),
                }),
            };
            ListChanges::new(&bef_entries).with_diff(idx, diff).build()
        }
        None => {
            let new_entry = FeeEntry {
                mint,
                threshold_nanos: new_threshold,
                fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
                    base_fee: new_fees[0],
                    threshold_fee: new_fees[1],
                    max_fee: new_fees[2],
                    output_fee: new_fees[3],
                }),
            };
            // Find insert position in sorted bef_entries
            let ins_idx = bef_entries.partition_point(|e| e.mint < mint);
            let mut changes = Vec::new();
            for _ in 0..ins_idx {
                changes.push(ListChange::Diff(Default::default()));
            }
            changes.push(ListChange::Add(new_entry));
            for _ in ins_idx..bef_entries.len() {
                changes.push(ListChange::Diff(Default::default()));
            }
            changes
        }
    };

    assert_diffs_pricing_state(
        (Diff::Unchanged, changes),
        (bef_admin, &bef_entries),
        (aft_admin, &aft_entries),
    );
    assert_valid_fee_entries(&aft_entries);
}

fn set_fee_entry_err_test(
    keys: &SetFeeEntryKeysOwned,
    ps: Account,
    data: &SetFeeEntryIxData,
    expected_err: impl Into<jiminy_entrypoint::program_error::ProgramError>,
) {
    let ix = set_fee_entry_ix(keys, data);
    let accs = set_fee_entry_accs(keys, ps);
    let err = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
        .unwrap_err();
    assert_jiminy_prog_err(&err, expected_err);
}

fn any_fee_values() -> impl Strategy<Value = (ThresholdNanos, FeeEntryNanos<FeeNanos>)> {
    (
        1u32..=999_999_999,
        0u32..=1_000_000_000u32,
        0u32..=1_000_000_000u32,
        0u32..=1_000_000_000u32,
        0u32..=1_000_000_000u32,
    )
        .prop_map(|(t, a, b, c, d)| {
            let mut fees = [a, b, c];
            fees.sort_unstable();
            (
                ThresholdNanos::new(t).unwrap(),
                FeeEntryNanos::from_destr(FeeEntryNanosDestr {
                    base_fee: FeeNanos::new(fees[0]).unwrap(),
                    threshold_fee: FeeNanos::new(fees[1]).unwrap(),
                    max_fee: FeeNanos::new(fees[2]).unwrap(),
                    output_fee: FeeNanos::new(d).unwrap(),
                }),
            )
        })
}

fn set_fee_entry_update_strat(
) -> impl Strategy<Value = (SetFeeEntryKeysOwned, Account, SetFeeEntryIxData)> {
    (
        any_reserve_v2_pricing_state(0usize..4),
        any_fee_values(),
        any_normal_pk(),
    )
        .prop_flat_map(|((admin, entries), (t, fees), payer)| {
            let idx = (0..entries.len()).prop_map(move |i| i);
            (
                Just(admin),
                Just(entries),
                idx,
                Just(t),
                Just(fees),
                Just(payer),
            )
        })
        .prop_map(|(admin, entries, idx, t, fees, payer)| {
            let mint = entries[idx].mint;
            let ps = mock_reserve_v2_pricing_state_account(admin, &entries);
            let keys = SetFeeEntryIxAccsGen {
                pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                    pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                    admin,
                }),
                suf: SetFeeEntryIxSufAccs::from_destr(SetFeeEntryIxSufAccsDestr { mint, payer }),
                sys_prog: *CONST_KEYS_OWNED.sys_prog(),
            };
            (keys, ps, SetFeeEntryIxData::new(t, fees))
        })
}

fn set_fee_entry_insert_strat(
) -> impl Strategy<Value = (SetFeeEntryKeysOwned, Account, SetFeeEntryIxData)> {
    (
        any_reserve_v2_pricing_state(0usize..4),
        any_fee_values(),
        any_normal_pk(),
    )
        .prop_flat_map(|((admin, entries), (t, fees), payer)| {
            let entries_clone = entries.clone();
            let missing = any_normal_pk().prop_filter("not in entries", move |pk| {
                !entries_clone.iter().any(|e| e.mint == *pk)
            });
            (
                Just(admin),
                Just(entries),
                missing,
                Just(t),
                Just(fees),
                Just(payer),
            )
        })
        .prop_map(|(admin, entries, mint, t, fees, payer)| {
            let ps = mock_reserve_v2_pricing_state_account(admin, &entries);
            let keys = SetFeeEntryIxAccsGen {
                pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                    pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                    admin,
                }),
                suf: SetFeeEntryIxSufAccs::from_destr(SetFeeEntryIxSufAccsDestr { mint, payer }),
                sys_prog: *CONST_KEYS_OWNED.sys_prog(),
            };
            (keys, ps, SetFeeEntryIxData::new(t, fees))
        })
}

proptest! {
    #[test]
    fn set_fee_entry_update((keys, ps, data) in set_fee_entry_update_strat()) {
        silence_mollusk_logs();
        set_fee_entry_test(&keys, ps, &data);
    }
}

proptest! {
    #[test]
    fn set_fee_entry_insert((keys, ps, data) in set_fee_entry_insert_strat()) {
        silence_mollusk_logs();
        set_fee_entry_test(&keys, ps, &data);
    }
}

fn set_fee_entry_wrong_admin_strat(
) -> impl Strategy<Value = (SetFeeEntryKeysOwned, Account, SetFeeEntryIxData)> {
    (
        any_reserve_v2_pricing_state(0usize..4),
        any_fee_values(),
        any_normal_pk(),
    )
        .prop_flat_map(|((admin, entries), (t, fees), payer)| {
            let wrong = any_normal_pk().prop_filter("differs from stored", move |pk| *pk != admin);
            (
                Just(admin),
                Just(entries),
                wrong,
                Just(t),
                Just(fees),
                Just(payer),
            )
        })
        .prop_map(|(admin, entries, wrong_admin, t, fees, payer)| {
            let ps = mock_reserve_v2_pricing_state_account(admin, &entries);
            let keys = SetFeeEntryIxAccsGen {
                pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                    pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                    admin: wrong_admin,
                }),
                suf: SetFeeEntryIxSufAccs::from_destr(SetFeeEntryIxSufAccsDestr {
                    mint: entries[0].mint,
                    payer,
                }),
                sys_prog: *CONST_KEYS_OWNED.sys_prog(),
            };
            (keys, ps, SetFeeEntryIxData::new(t, fees))
        })
}

proptest! {
    #[test]
    fn set_fee_entry_wrong_admin((keys, ps, data) in set_fee_entry_wrong_admin_strat()) {
        silence_mollusk_logs();
        set_fee_entry_err_test(&keys, ps, &data, INVALID_ARGUMENT);
    }
}

proptest! {
    #[test]
    fn set_fee_entry_missing_sig((keys, ps, data) in set_fee_entry_update_strat()) {
        silence_mollusk_logs();
        let mut ix = set_fee_entry_ix(&keys, &data);
        ix.accounts[ADMIN_IX_PRE_ACCS_IDX_ADMIN].is_signer = false;
        let accs = set_fee_entry_accs(&keys, ps);
        let err = SVM
            .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
            .unwrap_err();
        assert_jiminy_prog_err(&err, MISSING_REQUIRED_SIGNATURE);
    }
}

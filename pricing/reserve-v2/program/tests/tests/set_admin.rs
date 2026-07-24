use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_of_acc_data_packed,
    instructions::admin::{
        common::{AdminIxPreAccs, AdminIxPreAccsDestr, ADMIN_IX_PRE_ACCS_IDX_ADMIN},
        set_admin::{
            SetAdminIxAccsGen, SetAdminIxData, SET_ADMIN_IX_IS_SIGNER, SET_ADMIN_IX_IS_WRITER,
        },
    },
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
};
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_reserve_v2_pricing_state, assert_diffs_pricing_state,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_reserve_v2_pricing_state_account,
    mollusk_exec, silence_mollusk_logs, AccountMap, Diff, ListChanges,
};
use jiminy_cpi::program_error::MISSING_REQUIRED_SIGNATURE;
use jiminy_entrypoint::program_error::INVALID_ARGUMENT;
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

type SetAdminKeysOwned = SetAdminIxAccsGen<[u8; 32]>;

fn set_admin_ix(keys: &SetAdminKeysOwned) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(
            keys.seq(),
            SET_ADMIN_IX_IS_SIGNER.seq(),
            SET_ADMIN_IX_IS_WRITER.seq(),
        ),
        data: SetAdminIxData::as_buf().into(),
    }
}

fn set_admin_accs(keys: &SetAdminKeysOwned, pricing_state: Account) -> AccountMap {
    let pre = AdminIxPreAccs(keys.pre.0.map(Pubkey::new_from_array));
    AccountMap::from([
        (*pre.pricing_state(), pricing_state),
        (*pre.admin(), Account::default()),
        (Pubkey::new_from_array(keys.new_admin), Account::default()),
    ])
}

proptest! {
    #[test]
    fn set_admin_success(
        (admin, ref entries) in any_reserve_v2_pricing_state(0usize..1),
        new_admin_pk in any_normal_pk(),
    ) {
        silence_mollusk_logs();
        let pricing_state = mock_reserve_v2_pricing_state_account(admin, entries);

        let keys = SetAdminIxAccsGen {
            pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                admin,
            }),
            new_admin: new_admin_pk,
        };

        let accs = set_admin_accs(&keys, pricing_state.clone());
        let ix = set_admin_ix(&keys);
        let ok = SVM
            .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
            .unwrap();

        let ps_pk = Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state());

        let ps_acc_bef_aft = acc_bef_aft(&ps_pk, &accs, &ok.resulting_accounts);
        let [(bef_admin, bef_entries), (aft_admin, aft_entries)] = ps_acc_bef_aft.map(|d| {
            let (admin, entries_packed) = pricing_state_of_acc_data_packed(&d.data).unwrap();
            (
                admin,
                entries_packed
                    .0
                    .iter()
                    .map(|e| e.into_fee_entry())
                    .collect::<Vec<_>>(),
            )
        });

        assert_diffs_pricing_state(
            (
                Diff::StrictChanged(admin, new_admin_pk),
                ListChanges::new(&bef_entries).build(),
            ),
            (bef_admin, &bef_entries),
            (aft_admin, &aft_entries),
        );
    }

    #[test]
    fn set_admin_wrong_admin(
        ((admin, ref entries), wrong_admin) in any_reserve_v2_pricing_state(0usize..1)
            .prop_flat_map(|(a, e)| {
                let wrong = any_normal_pk()
                    .prop_filter("differs from stored", move |pk| *pk != a);
                ((Just(a), Just(e)), wrong)
            }),
    ) {
        silence_mollusk_logs();
        let pricing_state = mock_reserve_v2_pricing_state_account(admin, entries);

        let keys = SetAdminIxAccsGen {
            pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                admin: wrong_admin,
            }),
            new_admin: [99u8; 32],
        };

        let accs = set_admin_accs(&keys, pricing_state);
        let ix = set_admin_ix(&keys);
        let err = SVM
            .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
            .unwrap_err();
        assert_jiminy_prog_err(&err, INVALID_ARGUMENT);
    }

    #[test]
    fn set_admin_missing_sig(
        (admin, ref entries) in any_reserve_v2_pricing_state(0usize..1),
        new_admin_pk in any_normal_pk(),
    ) {
        silence_mollusk_logs();
        let pricing_state = mock_reserve_v2_pricing_state_account(admin, entries);

        let keys = SetAdminIxAccsGen {
            pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                admin,
            }),
            new_admin: new_admin_pk,
        };

        let mut ix = set_admin_ix(&keys);
        ix.accounts[ADMIN_IX_PRE_ACCS_IDX_ADMIN].is_signer = false;

        let accs = set_admin_accs(&keys, pricing_state);
        let err = SVM
            .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
            .unwrap_err();
        assert_jiminy_prog_err(&err, MISSING_REQUIRED_SIGNATURE);
    }
}

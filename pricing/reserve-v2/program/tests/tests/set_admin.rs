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
    any_normal_pk, any_reserve_v2_pricing_state, assert_jiminy_prog_err,
    keys_signer_writable_to_metas, mollusk_exec, silence_mollusk_logs, AccountMap,
};
use jiminy_cpi::program_error::MISSING_REQUIRED_SIGNATURE;
use jiminy_entrypoint::program_error::INVALID_ARGUMENT;
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{assert_valid_fee_entries, SVM};

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
        (*pre.admin(), Default::default()),
        (Pubkey::new_from_array(keys.new_admin), Default::default()),
    ])
}

proptest! {
    #[test]
    fn set_admin_success(
        (pricing_state, current_admin) in any_reserve_v2_pricing_state(),
        new_admin_pk in any_normal_pk(),
    ) {
        silence_mollusk_logs();

        let keys = SetAdminIxAccsGen {
            pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                admin: current_admin,
            }),
            new_admin: new_admin_pk,
        };

        let accs = set_admin_accs(&keys, pricing_state);
        let ix = set_admin_ix(&keys);
        let ok = SVM
            .with(|mollusk| mollusk_exec(mollusk, &[ix], &accs))
            .unwrap();

        let ps_pk = Pubkey::new_from_array(*CONST_PDA_KEYS_OWNED.pricing_state());
        let ps_acc = ok.resulting_accounts.get(&ps_pk).unwrap();
        let (admin, entries) = pricing_state_of_acc_data_packed(&ps_acc.data).unwrap();
        assert_eq!(*admin, keys.new_admin);
        assert_valid_fee_entries(
            &entries
                .0
                .iter()
                .map(|e| e.into_fee_entry())
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn set_admin_wrong_admin(
        (pricing_state, _stored_admin, wrong_admin) in any_reserve_v2_pricing_state()
            .prop_flat_map(|(ps, admin)| {
                let wrong = any_normal_pk()
                    .prop_filter("must differ from stored admin", move |pk| *pk != admin);
                (Just(ps), Just(admin), wrong)
            }),
    ) {
        silence_mollusk_logs();

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
        (pricing_state, current_admin) in any_reserve_v2_pricing_state(),
        new_admin_pk in any_normal_pk(),
    ) {
        silence_mollusk_logs();

        let keys = SetAdminIxAccsGen {
            pre: AdminIxPreAccs::from_destr(AdminIxPreAccsDestr {
                pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
                admin: current_admin,
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

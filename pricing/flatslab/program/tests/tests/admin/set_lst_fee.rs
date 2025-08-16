use inf1_pp_flatslab_core::{
    accounts::Slab,
    errs::FlatSlabProgramErr,
    instructions::admin::set_lst_fee::{
        NewSetLstFeeIxAccsBuilder, SetLstFeeIxAccs, SetLstFeeIxArgs, SetLstFeeIxData,
        SetLstFeeIxKeysOwned, SET_LST_FEE_IX_ACCS_IDX_ADMIN, SET_LST_FEE_IX_IS_SIGNER,
        SET_LST_FEE_IX_IS_WRITER,
    },
    keys::SLAB_ID,
    typedefs::SlabEntryPacked,
    ID,
};
use inf1_pp_flatslab_program::SYS_PROG_ID;
use mollusk_svm::{
    program::keyed_account_for_system_program,
    result::{Check, InstructionResult, ProgramResult},
};
use proptest::prelude::*;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::{
        mollusk::{silence_mollusk_logs, MOLLUSK},
        props::{clean_valid_slab, rand_unknown_pk, slab_data, MAX_MINTS},
        solana::{keys_signer_writable_to_metas, slab_account, PkAccountTup},
        tests::should_fail_with_flatslab_prog_err,
    },
    tests::admin::{assert_slab_entry_on_slab, assert_valid_slab},
};

fn set_lst_fee_ix(keys: &SetLstFeeIxKeysOwned, args: SetLstFeeIxArgs) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_LST_FEE_IX_IS_SIGNER.0.iter(),
        SET_LST_FEE_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetLstFeeIxData::new(args).as_buf().into(),
    }
}

fn set_lst_fee_ix_accounts(
    keys: &SetLstFeeIxKeysOwned,
    slab_data: Vec<u8>,
) -> SetLstFeeIxAccs<PkAccountTup> {
    NewSetLstFeeIxAccsBuilder::start()
        .with_slab((
            Pubkey::new_from_array(*keys.slab()),
            slab_account(slab_data),
        ))
        .with_admin((Pubkey::new_from_array(*keys.admin()), Default::default()))
        .with_mint((Pubkey::new_from_array(*keys.mint()), Default::default()))
        .with_payer((
            Pubkey::new_from_array(*keys.payer()),
            Account {
                // more than enough lamports to pay for any rent shortfall
                lamports: u64::MAX,
                ..Default::default()
            },
        ))
        .with_system_program((
            Pubkey::new_from_array(*keys.system_program()),
            keyed_account_for_system_program().1,
        ))
        .build()
}

fn assert_old_slab_entries_untouched(old_slab_data: &[u8], new_slab_data: &[u8]) {
    let old = Slab::of_acc_data(old_slab_data).unwrap().entries();
    for old_e in old.0 {
        assert_slab_entry_on_slab(new_slab_data, old_e);
    }
}

proptest! {
    #[test]
    fn set_lst_fee_success(
        slab in slab_data(0..=MAX_MINTS),
        payer in rand_unknown_pk(),
        mint in rand_unknown_pk(),
        inp_fee_nanos: i32,
        out_fee_nanos: i32,
    ) {
        silence_mollusk_logs();

        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewSetLstFeeIxAccsBuilder::start()
            .with_admin(admin)
            .with_mint(mint)
            .with_payer(payer)
            .with_system_program(SYS_PROG_ID)
            .with_slab(SLAB_ID)
            .build();
        let ix = set_lst_fee_ix(&keys, SetLstFeeIxArgs { inp_fee_nanos, out_fee_nanos });
        let accs = set_lst_fee_ix_accounts(&keys, slab.clone());
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
            let (_, new_slab) = resulting_accounts
                .iter()
                .find(|(pk, _)| *pk.as_array() == SLAB_ID)
                .unwrap();
            assert_valid_slab(&new_slab.data);

            let mut expected = SlabEntryPacked::DEFAULT;
            *expected.mint_mut() = mint;
            expected.set_inp_fee_nanos(inp_fee_nanos);
            expected.set_out_fee_nanos(out_fee_nanos);
            assert_slab_entry_on_slab(&new_slab.data, &expected);

            assert_old_slab_entries_untouched(&slab, &new_slab.data);
        });
    }
}

proptest! {
    #[test]
    fn set_lst_fee_fails_if_no_sig(
        slab in slab_data(0..=MAX_MINTS),
        payer in rand_unknown_pk(),
        mint in rand_unknown_pk(),
        inp_fee_nanos: i32,
        out_fee_nanos: i32,
    ) {
        silence_mollusk_logs();

        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        let keys = NewSetLstFeeIxAccsBuilder::start()
            .with_admin(admin)
            .with_mint(mint)
            .with_payer(payer)
            .with_system_program(SYS_PROG_ID)
            .with_slab(SLAB_ID)
            .build();
        let mut ix = set_lst_fee_ix(&keys, SetLstFeeIxArgs { inp_fee_nanos, out_fee_nanos });
        ix.accounts[SET_LST_FEE_IX_ACCS_IDX_ADMIN].is_signer = false;
        let accs = set_lst_fee_ix_accounts(&keys, slab);
        should_fail_with_flatslab_prog_err(&ix, &accs.0, FlatSlabProgramErr::MissingAdminSignature);
    }
}

proptest! {
    #[test]
    fn set_lst_fee_fails_if_wrong_admin(
        slab in slab_data(0..=MAX_MINTS),
        wrong_admin: [u8; 32],
        payer in rand_unknown_pk(),
        mint in rand_unknown_pk(),
        inp_fee_nanos: i32,
        out_fee_nanos: i32,
    ) {
        let admin = *Slab::of_acc_data(&slab).unwrap().admin();
        if wrong_admin == admin {
            return Ok(());
        }

        silence_mollusk_logs();

        let keys = NewSetLstFeeIxAccsBuilder::start()
            .with_admin(wrong_admin)
            .with_mint(mint)
            .with_payer(payer)
            .with_system_program(SYS_PROG_ID)
            .with_slab(SLAB_ID)
            .build();
        let ix = set_lst_fee_ix(&keys, SetLstFeeIxArgs { inp_fee_nanos, out_fee_nanos });
        let accs = set_lst_fee_ix_accounts(&keys, slab);
        should_fail_with_flatslab_prog_err(&ix, &accs.0, FlatSlabProgramErr::MissingAdminSignature);
    }
}

/// Check that SetLstFee dont take up way too many CUs
/// (otherwise we might brick the acc if >1.4M CUs to edit account)
#[test]
fn set_lst_fee_cu_upper_limit() {
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
    let slab_data = clean_valid_slab(bytes);
    let args = SetLstFeeIxArgs {
        inp_fee_nanos: 1_000_000_000,
        out_fee_nanos: 1_000_000_000,
    };
    // worst-case: adding entry at start of array means
    // having to shift entire array right
    let slab = Slab::of_acc_data(&slab_data).unwrap();
    let entries = slab.entries().0;
    if *entries[0].mint() <= SMALLEST_MINT {
        return;
    }

    let admin = *slab.admin();
    let keys = NewSetLstFeeIxAccsBuilder::start()
        .with_admin(admin)
        .with_mint(SMALLEST_MINT)
        .with_payer(Pubkey::new_unique().to_bytes())
        .with_system_program(SYS_PROG_ID)
        .with_slab(SLAB_ID)
        .build();
    let ix = set_lst_fee_ix(&keys, args);
    let accs = set_lst_fee_ix_accounts(&keys, slab_data);

    MOLLUSK.with(|mollusk| {
        let InstructionResult {
            compute_units_consumed,
            ..
        } = mollusk.process_instruction(&ix, &accs.0);
        assert!(
            compute_units_consumed < CU_UPPER_LIMIT,
            "{compute_units_consumed}"
        );
    });
}

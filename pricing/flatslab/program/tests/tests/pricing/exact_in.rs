use inf1_pp_core::{instructions::IxArgs, pair::Pair, traits::main::PriceExactIn};
use inf1_pp_flatslab_core::{
    accounts::Slab, errs::FlatSlabProgramErr, pricing::FlatSlabPricingErr,
    typedefs::MintNotFoundErr,
};
use inf1_pp_flatslab_program::CustomProgErr;
use inf1_test_utils::{assert_prog_err_eq, silence_mollusk_logs, AccountMap};
use jiminy_entrypoint::program_error::ProgramError;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;

use crate::{
    common::{
        mollusk::SVM,
        props::{clean_valid_slab, non_slab_pks, slab_for_swap, MAX_MINTS},
        tests::should_fail_with_flatslab_prog_err,
    },
    tests::pricing::{price_exact_in_ix, price_ix_accounts, price_keys_owned},
};

proptest! {
    #[test]
    fn behaviour_should_be_same_as_lib(
        (slab_data, pair, pricing) in slab_for_swap(MAX_MINTS),
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();

        let args = IxArgs { amt, sol_value };
        SVM.with(|mollusk| {
            let keys = price_keys_owned(pair);
            let ix = price_exact_in_ix(args, &keys);
            let accs = price_ix_accounts(&keys, slab_data);
            let InstructionResult { program_result, return_data, .. } = mollusk.process_instruction(
                &ix,
                &accs.seq().cloned().collect::<Vec<_>>(),
            );
            let lib_res = pricing.price_exact_in(args);

            match (program_result, lib_res) {
                (ProgramResult::Success, Ok(lib_res)) => {
                    prop_assert_eq!(lib_res, u64::from_le_bytes(return_data.try_into().unwrap()));
                }
                (ProgramResult::Failure(e), Err(FlatSlabPricingErr::Ratio)) => {
                    assert_prog_err_eq(
                        &e,
                        &ProgramError::from(CustomProgErr(FlatSlabProgramErr::Pricing(FlatSlabPricingErr::Ratio)))
                    );
                },
                (ProgramResult::Failure(e), Err(FlatSlabPricingErr::NetNegativeFees)) => {
                    assert_prog_err_eq(
                        &e,
                        &ProgramError::from(CustomProgErr(FlatSlabProgramErr::Pricing(FlatSlabPricingErr::NetNegativeFees)))
                    );
                },
                (a, b) => {
                    panic!("{a:#?}, {b:#?}");
                }
            }
            Ok(())
        }).unwrap();
    }
}

proptest! {
    #[test]
    fn should_fail_with_mint_not_found_for_unknown_mints(
        (slab_data, _, _) in slab_for_swap(MAX_MINTS),
        inp: [u8; 32],
        out: [u8; 32],
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();

        let slab = Slab::of_acc_data(&slab_data).unwrap();
        let entries = slab.entries();
        if entries.find_by_mint(&inp).is_ok() && entries.find_by_mint(&out).is_ok() {
            return Ok(());
        }

        let args = IxArgs { amt, sol_value };
        let keys = price_keys_owned(Pair { inp, out });
        let ix = price_exact_in_ix(args, &keys);
        let accs = price_ix_accounts(&keys, slab_data);
        should_fail_with_flatslab_prog_err(
            &ix,
            &accs.seq().cloned().collect::<AccountMap>(),
            FlatSlabProgramErr::MintNotFound(
                // dont-cares, just checking ProgramError code here
                MintNotFoundErr { expected_i: 0, mint: Default::default() }
            )
        );
    }
}

proptest! {
    #[test]
    fn should_fail_with_wrong_slab_acc_for_wrong_slab_acc(
        (slab_data, pair, _) in slab_for_swap(MAX_MINTS),
        wrong_slab_acc in non_slab_pks(),
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();

        let args = IxArgs { amt, sol_value };
        let mut keys = price_keys_owned(pair);
        keys.suf.0.set_slab(wrong_slab_acc);

        let ix = price_exact_in_ix(args, &keys);
        let accs = price_ix_accounts(&keys, slab_data);
        should_fail_with_flatslab_prog_err(
            &ix,
            &accs.seq().cloned().collect::<AccountMap>(),
            FlatSlabProgramErr::WrongSlabAcc,
        );
    }
}

/// Check that pricing instructions dont take up way too many CUs
/// (for binary search for large Slabs)
/// since this affects composability
#[test]
fn price_exact_in_cu_upper_limit() {
    const CU_UPPER_LIMIT: u64 = 2_000;
    const N_ENTRIES: usize = 100_000;

    silence_mollusk_logs();

    let mut rng = rand::rng();
    let mut bytes = vec![0u8; Slab::account_size(N_ENTRIES)];
    rng.fill_bytes(&mut bytes);
    let slab_data = clean_valid_slab(bytes);
    let args = IxArgs {
        amt: 1_000_000_000,
        sol_value: 1_000_000_000,
    };
    // (one of) binary search's worst case is the start/end of array
    let slab = Slab::of_acc_data(&slab_data).unwrap();
    let entries = slab.entries().0;
    let pair = Pair {
        inp: *entries.first().unwrap().mint(),
        out: *entries.last().unwrap().mint(),
    };
    SVM.with(|mollusk| {
        let keys = price_keys_owned(pair);
        let ix = price_exact_in_ix(args, &keys);
        let accs = price_ix_accounts(&keys, slab_data);
        // invocation might fail with PricingError
        let InstructionResult {
            compute_units_consumed,
            ..
        } = mollusk.process_instruction(&ix, &accs.seq().cloned().collect::<Vec<_>>());
        assert!(
            compute_units_consumed < CU_UPPER_LIMIT,
            "{compute_units_consumed}"
        );
    });
}

use inf1_pp_core::{
    instructions::{
        price::exact_out::{
            price_exact_out_ix_is_signer, price_exact_out_ix_is_writer,
            price_exact_out_ix_keys_owned, PriceExactOutIxData,
        },
        IxArgs,
    },
    pair::Pair,
    traits::main::PriceExactOut,
};
use inf1_pp_flatslab_core::{
    accounts::Slab, errs::FlatSlabProgramErr, pricing::FlatSlabPricingErr,
    typedefs::MintNotFoundErr, ID,
};
use inf1_pp_flatslab_program::CustomProgErr;
use inf1_test_utils::{assert_prog_err_eq, keys_signer_writable_to_metas, silence_mollusk_logs};
use jiminy_entrypoint::program_error::ProgramError;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::{
        mollusk::SVM,
        props::{non_slab_pks, slab_for_swap, MAX_MINTS},
        tests::should_fail_with_flatslab_prog_err,
    },
    tests::pricing::{price_ix_accounts, price_keys_owned, PriceIxKeysOwned},
};

fn price_exact_out_ix(args: IxArgs, keys: &PriceIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        price_exact_out_ix_keys_owned(keys).seq(),
        price_exact_out_ix_is_signer(keys).seq(),
        price_exact_out_ix_is_writer(keys).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: PriceExactOutIxData::new(args).as_buf().into(),
    }
}

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
            let ix = price_exact_out_ix(args, &keys);
            let accs = price_ix_accounts(&keys, slab_data);
            let InstructionResult { program_result, return_data, .. } = mollusk.process_instruction(
                &ix,
                &accs.seq().cloned().collect::<Vec<_>>(),
            );
            let lib_res = pricing.price_exact_out(args);
            match (program_result, lib_res) {
                (ProgramResult::Success, Ok(lib_res)) => {
                    prop_assert_eq!(lib_res, u64::from_le_bytes(return_data.try_into().unwrap()));
                }
                (ProgramResult::Failure(e), Err(_)) => {
                    assert_prog_err_eq(
                        &e,
                        &ProgramError::from(CustomProgErr(FlatSlabProgramErr::Pricing(FlatSlabPricingErr::Ratio)))
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
        let ix = price_exact_out_ix(args, &keys);
        let accs = price_ix_accounts(&keys, slab_data);
        should_fail_with_flatslab_prog_err(
            &ix,
            &accs.seq().cloned().collect::<Vec<_>>(),
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

        let ix = price_exact_out_ix(args, &keys);
        let accs = price_ix_accounts(&keys, slab_data);
        should_fail_with_flatslab_prog_err(
            &ix,
            &accs.seq().cloned().collect::<Vec<_>>(),
            FlatSlabProgramErr::WrongSlabAcc,
        );
    }
}

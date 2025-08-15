use inf1_pp_core::{
    instructions::{
        price::{
            exact_in::{
                price_exact_in_ix_is_signer, price_exact_in_ix_is_writer,
                price_exact_in_ix_keys_owned, PriceExactInIxData,
            },
            IxAccs, NewIxPreAccsBuilder,
        },
        IxArgs,
    },
    pair::Pair,
    traits::main::PriceExactIn,
};
use inf1_pp_flatslab_core::{
    errs::FlatSlabProgramErr, instructions::pricing::FlatSlabPpAccs, keys::SLAB_ID,
    pricing::FlatSlabPricingErr, ID,
};
use inf1_pp_flatslab_program::CustomProgErr;
use jiminy_entrypoint::program_error::ProgramError;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::{
        mollusk::{silence_mollusk_logs, MOLLUSK},
        props::slab_for_swap,
        solana::{assert_prog_err_eq, keys_signer_writable_to_metas},
    },
    tests::pricing::ix_accs,
};

fn price_exact_in_ix(Pair { inp, out }: Pair<[u8; 32]>, args: IxArgs) -> Instruction {
    let suf = FlatSlabPpAccs::new(SLAB_ID);
    let accs = IxAccs {
        ix_prefix: NewIxPreAccsBuilder::start()
            .with_input_mint(inp)
            .with_output_mint(out)
            .build(),
        suf,
    };
    let accounts = keys_signer_writable_to_metas(
        price_exact_in_ix_keys_owned(&accs).seq(),
        price_exact_in_ix_is_signer(&accs).seq(),
        price_exact_in_ix_is_writer(&accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: PriceExactInIxData::new(args).as_buf().into(),
    }
}

proptest! {
    #[test]
    fn behaviour_should_be_same_as_lib(
        (slab_data, pair, pricing) in slab_for_swap(),
        amt: u64,
        sol_value: u64,
    ) {
        silence_mollusk_logs();
        let args = IxArgs { amt, sol_value };
        MOLLUSK.with(|mollusk| {
            let ix = price_exact_in_ix(pair, args);
            let InstructionResult { program_result, return_data, .. } = mollusk.process_instruction(
                &ix,
                &ix_accs(&ix, slab_data),
            );
            let lib_res = pricing.price_exact_in(args);
            match (program_result, lib_res) {
                (ProgramResult::Success, Ok(lib_res)) => {
                    prop_assert_eq!(lib_res, u64::from_le_bytes(return_data.try_into().unwrap()));
                }
                (ProgramResult::Failure(e), Err(_)) => {
                    assert_prog_err_eq(
                        e,
                        ProgramError::from(CustomProgErr(FlatSlabProgramErr::Pricing(FlatSlabPricingErr::Ratio)))
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

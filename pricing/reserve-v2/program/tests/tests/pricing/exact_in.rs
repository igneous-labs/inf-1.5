use inf1_pp_core::{
    instructions::{
        price::exact_in::{
            price_exact_in_ix_is_signer, price_exact_in_ix_is_writer, price_exact_in_ix_keys_owned,
            PriceExactInIxData,
        },
        IxArgs,
    },
    pair::Pair,
    traits::main::{PriceExactIn, PriceExactOut},
};
use inf1_pp_reserve_v2_core::{
    errs::{OverCapErr, ReserveV2ProgramErr},
    keys::CONST_KEYS_OWNED,
    pricing::RangeOutPricing,
};
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;
use inf1_test_utils::keys_signer_writable_to_metas;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::tests::pricing::{
    execute_failure, execute_success, lp_entry, lst_entry, price_ix_accounts, price_keys_owned,
    pricing_state_account, valid_accounts, wsol_entry, PriceIxKeysOwned, LAMPORTS_PER_SOL,
    POOL_SOL_VALUE, WSOL_BALANCE,
};

fn price_exact_in_ix(args: IxArgs, keys: &PriceIxKeysOwned) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(
            price_exact_in_ix_keys_owned(keys).seq(),
            price_exact_in_ix_is_signer(keys).seq(),
            price_exact_in_ix_is_writer(keys).seq(),
        ),
        data: PriceExactInIxData::new(args).as_buf().into(),
    }
}

#[test]
fn flat_route_success() {
    let input_entry = wsol_entry();
    let output_entry = lp_entry();
    let keys = price_keys_owned(Pair {
        inp: input_entry.mint,
        out: output_entry.mint,
    });
    let args = IxArgs {
        amt: 123,
        sol_value: LAMPORTS_PER_SOL,
    };
    let accounts = price_ix_accounts(
        &keys,
        pricing_state_account(vec![input_entry, output_entry]),
        Account::default(),
        Account::default(),
    );

    execute_success(price_exact_in_ix(args, &keys), &accounts, args.sol_value);
}

#[test]
fn range_out_route_success() {
    let input_entry = lst_entry();
    let output_entry = wsol_entry();
    let keys = price_keys_owned(Pair {
        inp: input_entry.mint,
        out: output_entry.mint,
    });
    let args = IxArgs {
        amt: 456,
        sol_value: LAMPORTS_PER_SOL / 2,
    };
    let expected =
        RangeOutPricing::from_entries(&input_entry, &output_entry, POOL_SOL_VALUE, WSOL_BALANCE)
            .price_exact_in(args)
            .unwrap();
    let accounts = valid_accounts(&keys, vec![lp_entry(), input_entry, output_entry]);

    execute_success(price_exact_in_ix(args, &keys), &accounts, expected);
}

#[test]
fn range_out_over_cap_fails() {
    let input_entry = lst_entry();
    let output_entry = wsol_entry();
    let keys = price_keys_owned(Pair {
        inp: input_entry.mint,
        out: output_entry.mint,
    });
    let pricing =
        RangeOutPricing::from_entries(&input_entry, &output_entry, POOL_SOL_VALUE, WSOL_BALANCE);
    let full_drain_cost = pricing
        .price_exact_out(IxArgs {
            amt: 0,
            sol_value: WSOL_BALANCE,
        })
        .unwrap();
    let args = IxArgs {
        amt: 0,
        sol_value: full_drain_cost + 1,
    };
    let accounts = valid_accounts(&keys, vec![lp_entry(), input_entry, output_entry]);

    execute_failure(
        price_exact_in_ix(args, &keys),
        &accounts,
        CustomProgErr(ReserveV2ProgramErr::OverCap(OverCapErr {
            requested_out_sol_value: WSOL_BALANCE + 1,
            wsol_balance: WSOL_BALANCE,
        })),
    );
}

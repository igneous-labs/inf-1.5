use inf1_ctl_jiminy::{err::Inf1CtlErr, program_err::Inf1CtlCustomProgErr};
use inf1_pp_core::{
    instructions::{
        price::exact_out::{
            price_exact_out_ix_is_signer, price_exact_out_ix_is_writer,
            price_exact_out_ix_keys_owned, PriceExactOutIxData, PRICE_EXACT_OUT_IX_DISCM,
        },
        IxArgs,
    },
    pair::Pair,
    traits::main::PriceExactOut,
};
use inf1_pp_reserve_v2_core::{
    errs::{OverCapErr, ReserveV2ProgramErr, SameMintErr, WsolBalanceGtPoolSolValueErr},
    keys::CONST_KEYS_OWNED,
    pricing::{FlatPricing, RangeOutPricing},
};
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;
use inf1_test_utils::keys_signer_writable_to_metas;
use jiminy_cpi::program_error::{INVALID_ACCOUNT_DATA, INVALID_ARGUMENT, INVALID_INSTRUCTION_DATA};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::tests::pricing::{
    execute_failure, execute_success, lp_entry, lst_entry, pool_state_account, price_ix_accounts,
    price_keys_owned, pricing_state_account, valid_accounts, wsol_entry, wsol_reserves_account,
    PriceIxKeysOwned, LAMPORTS_PER_SOL, LST_MINT, POOL_SOL_VALUE, WSOL_BALANCE,
};

fn price_exact_out_ix(args: IxArgs, keys: &PriceIxKeysOwned) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(
            price_exact_out_ix_keys_owned(keys).seq(),
            price_exact_out_ix_is_signer(keys).seq(),
            price_exact_out_ix_is_writer(keys).seq(),
        ),
        data: PriceExactOutIxData::new(args).as_buf().into(),
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
    let expected = FlatPricing::from_entries(&input_entry, &output_entry)
        .price_exact_out(args)
        .unwrap();
    let accounts = price_ix_accounts(
        &keys,
        pricing_state_account(vec![input_entry, output_entry]),
        Account::default(),
        Account::default(),
    );

    execute_success(price_exact_out_ix(args, &keys), &accounts, expected);
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
            .price_exact_out(args)
            .unwrap();
    let accounts = valid_accounts(&keys, vec![lp_entry(), input_entry, output_entry]);

    execute_success(price_exact_out_ix(args, &keys), &accounts, expected);
}

#[test]
fn missing_input_entry_fails() {
    let keys = price_keys_owned(Pair {
        inp: LST_MINT,
        out: *CONST_KEYS_OWNED.wsol_mint(),
    });
    let args = IxArgs::default();
    let accounts = valid_accounts(&keys, vec![lp_entry(), wsol_entry()]);

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        CustomProgErr(ReserveV2ProgramErr::MintNotFound(
            inf1_pp_reserve_v2_core::typedefs::MintNotFoundErr {
                expected_i: 0,
                mint: LST_MINT,
            },
        )),
    );
}

#[test]
fn missing_output_entry_fails() {
    let keys = price_keys_owned(Pair {
        inp: *CONST_KEYS_OWNED.wsol_mint(),
        out: LST_MINT,
    });
    let args = IxArgs::default();
    let accounts = valid_accounts(&keys, vec![lp_entry(), wsol_entry()]);

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        CustomProgErr(ReserveV2ProgramErr::MintNotFound(
            inf1_pp_reserve_v2_core::typedefs::MintNotFoundErr {
                expected_i: 0,
                mint: LST_MINT,
            },
        )),
    );
}

#[test]
fn same_mint_fails() {
    let mint = *CONST_KEYS_OWNED.wsol_mint();
    let keys = price_keys_owned(Pair {
        inp: mint,
        out: mint,
    });
    let args = IxArgs::default();
    let accounts = price_ix_accounts(
        &keys,
        Account::default(),
        Account::default(),
        Account::default(),
    );

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        CustomProgErr(ReserveV2ProgramErr::SameMint(SameMintErr { mint })),
    );
}

#[test]
fn range_out_over_cap_fails() {
    let keys = price_keys_owned(Pair {
        inp: LST_MINT,
        out: *CONST_KEYS_OWNED.wsol_mint(),
    });
    let args = IxArgs {
        amt: 0,
        sol_value: WSOL_BALANCE + 1,
    };
    let accounts = valid_accounts(&keys, vec![lp_entry(), lst_entry(), wsol_entry()]);

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        CustomProgErr(ReserveV2ProgramErr::OverCap(OverCapErr {
            requested_out_sol_value: args.sol_value,
            wsol_balance: WSOL_BALANCE,
        })),
    );
}

#[test]
fn zero_pool_sol_value_fails_for_range_out() {
    let keys = price_keys_owned(Pair {
        inp: LST_MINT,
        out: *CONST_KEYS_OWNED.wsol_mint(),
    });
    let args = IxArgs::default();
    let accounts = price_ix_accounts(
        &keys,
        pricing_state_account(vec![lp_entry(), lst_entry(), wsol_entry()]),
        pool_state_account(0),
        wsol_reserves_account(0),
    );

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        CustomProgErr(ReserveV2ProgramErr::ZeroPoolSolValue),
    );
}

#[test]
fn wsol_balance_gt_pool_sol_value_fails_for_range_out() {
    let keys = price_keys_owned(Pair {
        inp: LST_MINT,
        out: *CONST_KEYS_OWNED.wsol_mint(),
    });
    let args = IxArgs::default();
    let wsol_balance = POOL_SOL_VALUE + 1;
    let accounts = price_ix_accounts(
        &keys,
        pricing_state_account(vec![lp_entry(), lst_entry(), wsol_entry()]),
        pool_state_account(POOL_SOL_VALUE),
        wsol_reserves_account(wsol_balance),
    );

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        CustomProgErr(ReserveV2ProgramErr::WsolBalanceGtPoolSolValue(
            WsolBalanceGtPoolSolValueErr {
                pool_sol_value: POOL_SOL_VALUE,
                wsol_balance,
            },
        )),
    );
}

#[test]
fn malformed_pool_state_fails_for_range_out() {
    let keys = price_keys_owned(Pair {
        inp: LST_MINT,
        out: *CONST_KEYS_OWNED.wsol_mint(),
    });
    let args = IxArgs::default();
    let accounts = price_ix_accounts(
        &keys,
        pricing_state_account(vec![lp_entry(), lst_entry(), wsol_entry()]),
        Account::default(),
        wsol_reserves_account(WSOL_BALANCE),
    );

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData),
    );
}

#[test]
fn malformed_wsol_reserves_fails_for_range_out() {
    let keys = price_keys_owned(Pair {
        inp: LST_MINT,
        out: *CONST_KEYS_OWNED.wsol_mint(),
    });
    let args = IxArgs::default();
    let accounts = price_ix_accounts(
        &keys,
        pricing_state_account(vec![lp_entry(), lst_entry(), wsol_entry()]),
        pool_state_account(POOL_SOL_VALUE),
        Account::default(),
    );

    execute_failure(
        price_exact_out_ix(args, &keys),
        &accounts,
        INVALID_ACCOUNT_DATA,
    );
}

#[test]
fn wrong_fixed_suffix_account_fails() {
    let mut keys = price_keys_owned(Pair {
        inp: *CONST_KEYS_OWNED.wsol_mint(),
        out: *CONST_KEYS_OWNED.lp_mint(),
    });
    keys.suf.0.set_pool_state([9; 32]);
    let args = IxArgs::default();
    let accounts = valid_accounts(&keys, vec![lp_entry(), wsol_entry()]);

    execute_failure(price_exact_out_ix(args, &keys), &accounts, INVALID_ARGUMENT);
}

#[test]
fn malformed_instruction_data_fails() {
    let keys = price_keys_owned(Pair {
        inp: *CONST_KEYS_OWNED.wsol_mint(),
        out: *CONST_KEYS_OWNED.lp_mint(),
    });
    let mut ix = price_exact_out_ix(IxArgs::default(), &keys);
    ix.data = vec![PRICE_EXACT_OUT_IX_DISCM];
    let accounts = valid_accounts(&keys, vec![lp_entry(), wsol_entry()]);

    execute_failure(ix, &accounts, INVALID_INSTRUCTION_DATA);
}

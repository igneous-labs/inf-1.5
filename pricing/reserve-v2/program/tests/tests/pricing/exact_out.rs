use crate::{
    common::{SVM, SVM_MUT},
    tests::pricing::{
        pool_state_account, price_ix_accounts, price_keys_owned, pricing_state_account,
        wsol_reserves_account, PriceIxKeysOwned,
    },
};
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2, err::Inf1CtlErr, program_err::Inf1CtlCustomProgErr,
};
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
    errs::{OverCapErr, ReserveV2ProgramErr, SameMintErr},
    instructions::pricing::IxSufAccs,
    keys::CONST_KEYS_OWNED,
    pricing::{FlatPricing, RangeOutPricing},
    typedefs::{FeeEntry, FeeEntryNanos, FeeEntryNanosDestr},
};
use inf1_pp_reserve_v2_jiminy::program_err::CustomProgErr;
use inf1_test_utils::{
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mollusk_exec,
    mollusk_with_clock_override, pool_state_v2_account, AccountMap, ClockArgs, ClockU64s,
};
use jiminy_cpi::program_error::{INVALID_ACCOUNT_DATA, INVALID_ARGUMENT, INVALID_INSTRUCTION_DATA};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

const LST_MINT: [u8; 32] = [7; 32];
const THRESHOLD_NANOS: u32 = 500_000_000;
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
const POOL_SOL_VALUE: u64 = 10 * LAMPORTS_PER_SOL;
const WSOL_BALANCE: u64 = 7 * LAMPORTS_PER_SOL;

fn wsol_entry() -> inf1_pp_reserve_v2_core::typedefs::FeeEntry {
    FeeEntry {
        mint: *CONST_KEYS_OWNED.wsol_mint(),
        threshold_nanos: THRESHOLD_NANOS,
        fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
            base_fee: 0,
            threshold_fee: 0,
            max_fee: 0,
            output_fee: 0,
        }),
    }
}

fn lp_entry() -> inf1_pp_reserve_v2_core::typedefs::FeeEntry {
    FeeEntry {
        mint: *CONST_KEYS_OWNED.lp_mint(),
        threshold_nanos: THRESHOLD_NANOS,
        fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
            base_fee: 75_000_000,
            threshold_fee: 75_000_000,
            max_fee: 75_000_000,
            output_fee: 0,
        }),
    }
}

fn lst_entry() -> inf1_pp_reserve_v2_core::typedefs::FeeEntry {
    FeeEntry {
        mint: LST_MINT,
        threshold_nanos: THRESHOLD_NANOS,
        fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
            base_fee: 100_000_000,
            threshold_fee: 200_000_000,
            max_fee: 300_000_000,
            output_fee: 1_000_000_000,
        }),
    }
}

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

fn valid_accounts(
    keys: &PriceIxKeysOwned,
    entries: Vec<inf1_pp_reserve_v2_core::typedefs::FeeEntry>,
) -> AccountMap {
    price_ix_accounts(
        keys,
        IxSufAccs::new([
            pricing_state_account(entries),
            pool_state_account(POOL_SOL_VALUE),
            wsol_reserves_account(WSOL_BALANCE),
        ]),
    )
}

fn execute_success(ix: Instruction, accounts: &AccountMap, expected: u64) {
    let ok = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], accounts))
        .unwrap();
    assert_eq!(
        u64::from_le_bytes(ok.return_data.try_into().unwrap()),
        expected
    );
}

fn execute_failure(
    ix: Instruction,
    accounts: &AccountMap,
    expected: impl Into<jiminy_entrypoint::program_error::ProgramError>,
) {
    let err = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], accounts))
        .unwrap_err();
    assert_jiminy_prog_err(&err, expected);
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
        IxSufAccs::new([
            pricing_state_account(vec![input_entry, output_entry]),
            Account::default(),
            Account::default(),
        ]),
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
fn range_out_uses_lookahead_depositor_sol_value() {
    const WITHHELD_LAMPORTS: u64 = 2 * LAMPORTS_PER_SOL;
    const PROTOCOL_FEE_LAMPORTS: u64 = LAMPORTS_PER_SOL;
    // depositor due starts at 10 - 2 - 1 = 7 SOL
    // 1 SOL released per slot, split 50/50 between depositors and protocol
    // projected depositor value becomes 7.5 SOL
    const PROJECTED_DEPOSITOR_SOL_VALUE: u64 = 15 * LAMPORTS_PER_SOL / 2;

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

    let mut pool_state = PoolStateV2::init(0, *CONST_KEYS_OWNED.lp_mint());
    pool_state.total_sol_value = POOL_SOL_VALUE;
    pool_state.withheld_lamports = WITHHELD_LAMPORTS;
    pool_state.protocol_fee_lamports = PROTOCOL_FEE_LAMPORTS;
    pool_state.protocol_fee_nanos = 500_000_000; // Protocol receives 50% of released yield
    pool_state.rps = 1 << 62; // Release 50% of withheld yield per slot

    let expected = RangeOutPricing::from_entries(
        &input_entry,
        &output_entry,
        PROJECTED_DEPOSITOR_SOL_VALUE,
        WSOL_BALANCE,
    )
    .price_exact_out(args)
    .unwrap();
    let accounts = price_ix_accounts(
        &keys,
        IxSufAccs::new([
            pricing_state_account(vec![lp_entry(), input_entry, output_entry]),
            pool_state_v2_account(pool_state),
            wsol_reserves_account(WSOL_BALANCE),
        ]),
    );
    let ix = price_exact_out_ix(args, &keys);

    let ok = SVM_MUT
        .with_borrow_mut(|mollusk| {
            mollusk_with_clock_override(
                mollusk,
                &ClockArgs {
                    u64s: ClockU64s::default().with_slot(Some(1)),
                    ..Default::default()
                },
                |mollusk| mollusk_exec(mollusk, &[ix], &accounts),
            )
        })
        .unwrap();
    assert_eq!(
        u64::from_le_bytes(ok.return_data.try_into().unwrap()),
        expected
    );
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
        IxSufAccs::new([Account::default(), Account::default(), Account::default()]),
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
fn malformed_pool_state_fails_for_range_out() {
    let keys = price_keys_owned(Pair {
        inp: LST_MINT,
        out: *CONST_KEYS_OWNED.wsol_mint(),
    });
    let args = IxArgs::default();
    let accounts = price_ix_accounts(
        &keys,
        IxSufAccs::new([
            pricing_state_account(vec![lp_entry(), lst_entry(), wsol_entry()]),
            Account::default(),
            wsol_reserves_account(WSOL_BALANCE),
        ]),
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
        IxSufAccs::new([
            pricing_state_account(vec![lp_entry(), lst_entry(), wsol_entry()]),
            pool_state_account(POOL_SOL_VALUE),
            Account::default(),
        ]),
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

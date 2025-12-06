use inf1_ctl_jiminy::{
    accounts::pool_state::{
        PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed, PoolStateV2U64s,
    },
    err::Inf1CtlErr,
    instructions::protocol_fee::withdraw_protocol_fees::v2::{
        NewWithdrawProtocolFeesV2IxAccsBuilder, WithdrawProtocolFeesV2IxData,
        WithdrawProtocolFeesV2IxKeysOwned, WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_BENEFICIARY,
        WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_INF_MINT,
        WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_POOL_STATE,
        WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_WITHDRAW_TO, WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER,
        WITHDRAW_PROTOCOL_FEES_V2_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    svc::InfCalc,
    typedefs::pool_sv::PoolSvLamports,
};
use inf1_svc_ag_core::inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM;
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state_v2, assert_diffs_pool_state_v2,
    assert_jiminy_prog_err, assert_token_acc_diffs, keys_signer_writable_to_metas,
    mock_mint_with_prog, mock_sys_acc, mock_token_acc_with_prog, mollusk_exec,
    pool_state_v2_account, pool_state_v2_u64s_just_lamports_strat,
    pool_state_v2_u8_bools_normal_strat, pool_sv_lamports_solvent_strat, raw_mint, raw_token_acc,
    silence_mollusk_logs, token_acc_bal_diff_changed, AccountMap, Diff, DiffsPoolStateV2,
    PoolStateV2FtaStrat, ALL_FIXTURES, INF_MINT,
};
use mollusk_svm::Mollusk;
use proptest::prelude::*;
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::{
    account::RawTokenAccount,
    mint::{Mint, RawMint},
};
use sanctum_u64_ratio::Ratio;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{header_lookahead_no_lsts, SVM};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};

const INF_MINT_ID: [u8; 32] = INF_MINT.to_bytes();

/// Safety margin to prevent u64 overflow in sol_to_inf calculation
/// when protocol_fee_lamports * inf_mint_supply
const SAFE_MUL_U64_MAX: u64 = u32::MAX as u64;

fn withdraw_protocol_fees_v2_ix(keys: &WithdrawProtocolFeesV2IxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER.0.iter(),
        WITHDRAW_PROTOCOL_FEES_V2_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: WithdrawProtocolFeesV2IxData::as_buf().into(),
    }
}

fn gen_inf_mint(supply: u64) -> RawMint {
    raw_mint(Some(POOL_STATE_ID), None, supply, 9)
}

fn withdraw_protocol_fees_v2_test_accs(
    keys: &WithdrawProtocolFeesV2IxKeysOwned,
    pool: PoolStateV2,
    inf_mint_supply: u64,
    withdraw_to_balance: u64,
) -> AccountMap {
    const LAMPORTS: u64 = 1_000_000_000;

    let token_prog = *keys.token_program();
    let inf_mint_pk = *keys.inf_mint();

    let accs = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
        .with_pool_state(pool_state_v2_account(pool))
        .with_beneficiary(mock_sys_acc(LAMPORTS))
        .with_withdraw_to(mock_token_acc_with_prog(
            raw_token_acc(inf_mint_pk, *keys.beneficiary(), withdraw_to_balance),
            token_prog,
        ))
        .with_inf_mint(mock_mint_with_prog(
            gen_inf_mint(inf_mint_supply),
            TOKENKEG_PROGRAM,
        ))
        .with_token_program(ALL_FIXTURES.get(&TOKENKEG_PROGRAM.into()).unwrap().clone())
        .build();

    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

fn withdraw_protocol_fees_v2_test(
    svm: &Mollusk,
    ix: Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let result = mollusk_exec(svm, std::slice::from_ref(&ix), bef);

    match expected_err {
        None => {
            let [pool_pk, withdraw_to_pk, inf_mint_pk] = [
                WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_POOL_STATE,
                WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_WITHDRAW_TO,
                WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_INF_MINT,
            ]
            .map(|i| ix.accounts[i].pubkey);
            let aft: AccountMap = result.unwrap().resulting_accounts;

            let [pool_state_bef, pool_state_aft] = {
                acc_bef_aft(&pool_pk, bef, &aft).map(|acc| {
                    PoolStateV2Packed::of_acc_data(&acc.data)
                        .unwrap()
                        .into_pool_state_v2()
                })
            };

            let pool_state_bef = header_lookahead_no_lsts(pool_state_bef, svm.sysvars.clock.slot);

            let [withdraw_to_bef, withdraw_to_aft] = {
                acc_bef_aft(&withdraw_to_pk, bef, &aft)
                    .map(|acc| RawTokenAccount::of_acc_data(&acc.data).unwrap())
            };

            let [inf_mint_bef, inf_mint_aft] = {
                acc_bef_aft(&inf_mint_pk, bef, &aft).map(|acc| {
                    RawMint::of_acc_data(&acc.data)
                        .and_then(Mint::try_from_raw)
                        .unwrap()
                })
            };

            let inf_calc = InfCalc {
                pool_lamports: PoolSvLamports::from_pool_state_v2(&pool_state_bef),
                mint_supply: inf_mint_bef.supply(),
            };
            let expected_minted = *inf_calc
                .sol_to_inf(pool_state_bef.protocol_fee_lamports)
                .unwrap()
                .start();

            assert_token_acc_diffs(
                withdraw_to_bef,
                withdraw_to_aft,
                &token_acc_bal_diff_changed(withdraw_to_bef, expected_minted as i128),
            );

            assert_eq!(
                inf_mint_aft.supply() - inf_mint_bef.supply(),
                expected_minted
            );

            assert_diffs_pool_state_v2(
                &DiffsPoolStateV2 {
                    u64s: PoolStateV2U64s::default().with_protocol_fee_lamports(Diff::Changed(
                        pool_state_bef.protocol_fee_lamports,
                        0,
                    )),
                    ..Default::default()
                },
                &pool_state_bef,
                &pool_state_aft,
            );

            // assert redemption rate does not change
            let [lp_due_bef, lp_due_aft] = [&pool_state_bef, &pool_state_aft].map(|ps| {
                PoolSvLamports::from_pool_state_v2(ps)
                    .lp_due_checked()
                    .unwrap()
            });

            let lp_rate_bef = Ratio {
                n: lp_due_bef,
                d: inf_mint_bef.supply(),
            };
            let lp_rate_aft_lenient = Ratio {
                n: lp_due_aft.saturating_add(2),
                d: inf_mint_aft.supply(),
            };

            assert!(
                lp_rate_aft_lenient >= lp_rate_bef,
                "LP redemption rate decreased: {:?} -> {:?}",
                lp_rate_bef,
                lp_rate_aft_lenient
            );
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
        }
    }
}

#[test]
fn withdraw_protocol_fees_v2_correct_basic() {
    const INF_MINT_SUPPLY: u64 = 10_000_000_000_000;
    const WITHDRAW_TO_BALANCE: u64 = 50;
    const PROTOCOL_FEE_LAMPORTS: u64 = 1_000_000_000;
    const TOTAL_SOL_VALUE: u64 = 100_000_000_000_000;

    // 69 + to avoid colliding with system prog
    let [ben, wt] = core::array::from_fn(|i| [69 + u8::try_from(i).unwrap(); 32]);

    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default()
            .with_protocol_fee_beneficiary(ben)
            .with_lp_token_mint(INF_MINT_ID),
        u64s: PoolStateV2U64s::default()
            .with_protocol_fee_lamports(PROTOCOL_FEE_LAMPORTS)
            .with_total_sol_value(TOTAL_SOL_VALUE),
        ..Default::default()
    }
    .into_pool_state_v2();

    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
        .with_pool_state(POOL_STATE_ID)
        .with_beneficiary(ben)
        .with_withdraw_to(wt)
        .with_inf_mint(INF_MINT_ID)
        .with_token_program(TOKENKEG_PROGRAM)
        .build();

    SVM.with(|svm| {
        withdraw_protocol_fees_v2_test(
            svm,
            withdraw_protocol_fees_v2_ix(&keys),
            &withdraw_protocol_fees_v2_test_accs(&keys, pool, INF_MINT_SUPPLY, WITHDRAW_TO_BALANCE),
            Option::<ProgramError>::None,
        );
    });
}

fn to_inp(
    (keys, pool, inf_mint_supply, withdraw_to_balance): (
        WithdrawProtocolFeesV2IxKeysOwned,
        PoolStateV2,
        u64,
        u64,
    ),
) -> (Instruction, AccountMap) {
    (
        withdraw_protocol_fees_v2_ix(&keys),
        withdraw_protocol_fees_v2_test_accs(&keys, pool, inf_mint_supply, withdraw_to_balance),
    )
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (0..=SAFE_MUL_U64_MAX)
        .prop_flat_map(|tsv| {
            pool_sv_lamports_solvent_strat(tsv)
                .prop_flat_map(|solvent_u64s| {
                    (
                        any_normal_pk(),
                        any_pool_state_v2(PoolStateV2FtaStrat {
                            u8_bools: pool_state_v2_u8_bools_normal_strat(),
                            addrs: PoolStateV2Addrs::default()
                                .with_lp_token_mint(Some(Just(INF_MINT_ID).boxed())),
                            u64s: pool_state_v2_u64s_just_lamports_strat(solvent_u64s)
                                .with_last_release_slot(Some(Just(0).boxed())),
                            ..Default::default()
                        }),
                        0..=SAFE_MUL_U64_MAX,
                        0..=u64::MAX,
                    )
                })
                .prop_map(|(wt_pk, ps, inf_mint_supply, withdraw_to_balance)| {
                    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
                        .with_pool_state(POOL_STATE_ID)
                        .with_beneficiary(ps.protocol_fee_beneficiary)
                        .with_withdraw_to(wt_pk)
                        .with_inf_mint(INF_MINT_ID)
                        .with_token_program(TOKENKEG_PROGRAM)
                        .build();

                    (keys, ps, inf_mint_supply, withdraw_to_balance)
                })
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_correct_pt(
      (ix, bef) in correct_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Option::<ProgramError>::None);
      });
  }
}

fn zero_fees_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (0..=SAFE_MUL_U64_MAX)
        .prop_flat_map(|tsv| {
            pool_sv_lamports_solvent_strat(tsv)
                .prop_flat_map(|solvent_u64s| {
                    (
                        any_normal_pk(),
                        any_pool_state_v2(PoolStateV2FtaStrat {
                            u8_bools: pool_state_v2_u8_bools_normal_strat(),
                            addrs: PoolStateV2Addrs::default()
                                .with_lp_token_mint(Some(Just(INF_MINT_ID).boxed())),
                            u64s: pool_state_v2_u64s_just_lamports_strat(solvent_u64s)
                                .with_protocol_fee_lamports(Some(Just(0).boxed()))
                                .with_last_release_slot(Some(Just(0).boxed())),
                            ..Default::default()
                        }),
                        0..=SAFE_MUL_U64_MAX,
                        0..=u64::MAX,
                    )
                })
                .prop_map(|(wt_pk, ps, inf_mint_supply, withdraw_to_balance)| {
                    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
                        .with_pool_state(POOL_STATE_ID)
                        .with_beneficiary(ps.protocol_fee_beneficiary)
                        .with_withdraw_to(wt_pk)
                        .with_inf_mint(INF_MINT_ID)
                        .with_token_program(TOKENKEG_PROGRAM)
                        .build();

                    (keys, ps, inf_mint_supply, withdraw_to_balance)
                })
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_zero_fees_pt(
      (ix, bef) in zero_fees_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Option::<ProgramError>::None);
      });
  }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (0..=SAFE_MUL_U64_MAX)
        .prop_flat_map(|tsv| {
            pool_sv_lamports_solvent_strat(tsv)
                .prop_flat_map(|solvent_u64s| {
                    (
                        any_pool_state_v2(PoolStateV2FtaStrat {
                            u8_bools: pool_state_v2_u8_bools_normal_strat(),
                            addrs: PoolStateV2Addrs::default()
                                .with_lp_token_mint(Some(Just(INF_MINT_ID).boxed())),
                            u64s: pool_state_v2_u64s_just_lamports_strat(solvent_u64s)
                                .with_last_release_slot(Some(Just(0).boxed())),
                            ..Default::default()
                        }),
                        0..=SAFE_MUL_U64_MAX,
                        0..=u64::MAX,
                    )
                })
                .prop_flat_map(|(ps, inf_mint_supply, withdraw_to_balance)| {
                    (
                        any_normal_pk().prop_filter("wrong beneficiary", move |pk| {
                            *pk != ps.protocol_fee_beneficiary
                        }),
                        any_normal_pk(),
                        Just(ps),
                        Just(inf_mint_supply),
                        Just(withdraw_to_balance),
                    )
                })
                .prop_map(
                    |(wrong_ben, wt_pk, ps, inf_mint_supply, withdraw_to_balance)| {
                        let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
                            .with_pool_state(POOL_STATE_ID)
                            .with_beneficiary(wrong_ben)
                            .with_withdraw_to(wt_pk)
                            .with_inf_mint(INF_MINT_ID)
                            .with_token_program(TOKENKEG_PROGRAM)
                            .build();

                        (keys, ps, inf_mint_supply, withdraw_to_balance)
                    },
                )
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_unauthorized_pt(
      (ix, bef) in unauthorized_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Some(INVALID_ARGUMENT));
      });
  }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, bef)| {
        ix.accounts[WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_IDX_BENEFICIARY].is_signer = false;
        (ix, bef)
    })
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_missing_sig_pt(
      (ix, bef) in missing_sig_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
      });
  }
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (0..=SAFE_MUL_U64_MAX)
        .prop_flat_map(|tsv| {
            pool_sv_lamports_solvent_strat(tsv)
                .prop_flat_map(|solvent_u64s| {
                    (
                        any_normal_pk(),
                        any_pool_state_v2(PoolStateV2FtaStrat {
                            u8_bools: pool_state_v2_u8_bools_normal_strat()
                                .with_is_disabled(Some(Just(true).boxed())),
                            addrs: PoolStateV2Addrs::default()
                                .with_lp_token_mint(Some(Just(INF_MINT_ID).boxed())),
                            u64s: pool_state_v2_u64s_just_lamports_strat(solvent_u64s)
                                .with_last_release_slot(Some(Just(0).boxed())),
                            ..Default::default()
                        }),
                        0..=SAFE_MUL_U64_MAX,
                        0..=u64::MAX,
                    )
                })
                .prop_map(|(wt_pk, ps, inf_mint_supply, withdraw_to_balance)| {
                    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
                        .with_pool_state(POOL_STATE_ID)
                        .with_beneficiary(ps.protocol_fee_beneficiary)
                        .with_withdraw_to(wt_pk)
                        .with_inf_mint(INF_MINT_ID)
                        .with_token_program(TOKENKEG_PROGRAM)
                        .build();

                    (keys, ps, inf_mint_supply, withdraw_to_balance)
                })
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_disabled_pt(
      (ix, bef) in disabled_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled)));
      });
  }
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (0..=SAFE_MUL_U64_MAX)
        .prop_flat_map(|tsv| {
            pool_sv_lamports_solvent_strat(tsv)
                .prop_flat_map(|solvent_u64s| {
                    (
                        any_normal_pk(),
                        any_pool_state_v2(PoolStateV2FtaStrat {
                            u8_bools: pool_state_v2_u8_bools_normal_strat()
                                .with_is_rebalancing(Some(Just(true).boxed())),
                            addrs: PoolStateV2Addrs::default()
                                .with_lp_token_mint(Some(Just(INF_MINT_ID).boxed())),
                            u64s: pool_state_v2_u64s_just_lamports_strat(solvent_u64s)
                                .with_last_release_slot(Some(Just(0).boxed())),
                            ..Default::default()
                        }),
                        0..=SAFE_MUL_U64_MAX,
                        0..=u64::MAX,
                    )
                })
                .prop_map(|(wt_pk, ps, inf_mint_supply, withdraw_to_balance)| {
                    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
                        .with_pool_state(POOL_STATE_ID)
                        .with_beneficiary(ps.protocol_fee_beneficiary)
                        .with_withdraw_to(wt_pk)
                        .with_inf_mint(INF_MINT_ID)
                        .with_token_program(TOKENKEG_PROGRAM)
                        .build();

                    (keys, ps, inf_mint_supply, withdraw_to_balance)
                })
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_rebalancing_pt(
      (ix, bef) in rebalancing_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing)));
      });
  }
}

fn wrong_token_prog_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (0..=SAFE_MUL_U64_MAX)
        .prop_flat_map(|tsv| {
            pool_sv_lamports_solvent_strat(tsv)
                .prop_flat_map(|solvent_u64s| {
                    (
                        any_normal_pk(),
                        any_normal_pk()
                            .prop_filter("must be wrong token prog", |pk| *pk != TOKENKEG_PROGRAM),
                        any_pool_state_v2(PoolStateV2FtaStrat {
                            u8_bools: pool_state_v2_u8_bools_normal_strat(),
                            addrs: PoolStateV2Addrs::default()
                                .with_lp_token_mint(Some(Just(INF_MINT_ID).boxed())),
                            u64s: pool_state_v2_u64s_just_lamports_strat(solvent_u64s)
                                .with_last_release_slot(Some(Just(0).boxed())),
                            ..Default::default()
                        }),
                        0..=SAFE_MUL_U64_MAX,
                        0..=u64::MAX,
                    )
                })
                .prop_map(
                    |(wt_pk, bad_token_prog, ps, inf_mint_supply, withdraw_to_balance)| {
                        let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
                            .with_pool_state(POOL_STATE_ID)
                            .with_beneficiary(ps.protocol_fee_beneficiary)
                            .with_withdraw_to(wt_pk)
                            .with_inf_mint(INF_MINT_ID)
                            .with_token_program(bad_token_prog)
                            .build();

                        (keys, ps, inf_mint_supply, withdraw_to_balance)
                    },
                )
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_wrong_token_prog_pt(
      (ix, bef) in wrong_token_prog_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Some(INVALID_ARGUMENT));
      });
  }
}

fn wrong_mint_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (0..=SAFE_MUL_U64_MAX)
        .prop_flat_map(|tsv| {
            (
                pool_sv_lamports_solvent_strat(tsv),
                any_normal_pk().prop_filter("mint must not match", |pk| *pk != INF_MINT_ID),
            )
                .prop_flat_map(|(solvent_u64s, wrong_mint)| {
                    (
                        any_normal_pk(),
                        any_pool_state_v2(PoolStateV2FtaStrat {
                            u8_bools: pool_state_v2_u8_bools_normal_strat(),
                            addrs: PoolStateV2Addrs::default()
                                .with_lp_token_mint(Some(Just(wrong_mint).boxed())),
                            u64s: pool_state_v2_u64s_just_lamports_strat(solvent_u64s)
                                .with_last_release_slot(Some(Just(0).boxed())),
                            ..Default::default()
                        }),
                        0..=SAFE_MUL_U64_MAX,
                        0..=u64::MAX,
                    )
                })
                .prop_map(|(wt_pk, ps, inf_mint_supply, withdraw_to_balance)| {
                    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
                        .with_pool_state(POOL_STATE_ID)
                        .with_beneficiary(ps.protocol_fee_beneficiary)
                        .with_withdraw_to(wt_pk)
                        .with_inf_mint(INF_MINT_ID)
                        .with_token_program(TOKENKEG_PROGRAM)
                        .build();

                    (keys, ps, inf_mint_supply, withdraw_to_balance)
                })
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn withdraw_protocol_fees_v2_wrong_mint_pt(
      (ix, bef) in wrong_mint_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          withdraw_protocol_fees_v2_test(svm, ix, &bef, Some(INVALID_ARGUMENT));
      });
  }
}

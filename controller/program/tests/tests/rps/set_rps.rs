use inf1_ctl_jiminy::{
    accounts::pool_state::{
        PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed, PoolStateV2U64s,
    },
    err::Inf1CtlErr,
    instructions::rps::set_rps::{
        NewSetRpsIxAccsBuilder, SetRpsIxData, SetRpsIxKeysOwned, SET_RPS_IX_ACCS_IDX_POOL_STATE,
        SET_RPS_IX_ACCS_IDX_RPS_AUTH, SET_RPS_IX_IS_SIGNER, SET_RPS_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    typedefs::{
        rps::{Rps, MIN_RPS_RAW},
        uq0f63::UQ0F63,
    },
    ID,
};
use inf1_svc_ag_core::calc::SvcCalcAg;
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state_v2, any_rps_strat, assert_diffs_pool_state_v2,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec,
    pool_state_v2_account, pool_state_v2_u8_bools_normal_strat, silence_mollusk_logs, AccountMap,
    Diff, DiffsPoolStateV2, PoolStateV2FtaStrat,
};
use jiminy_cpi::program_error::{
    ProgramError, INVALID_ARGUMENT, INVALID_INSTRUCTION_DATA, MISSING_REQUIRED_SIGNATURE,
};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use mollusk_svm::Mollusk;

use crate::common::{header_lookahead, Cbs, SVM};

fn pool_state_header_lookahead(ps: PoolStateV2, curr_slot: u64) -> PoolStateV2 {
    header_lookahead(ps, &[] as &[Cbs<SvcCalcAg>], curr_slot)
}

fn set_rps_ix(keys: SetRpsIxKeysOwned, rps: u64) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_RPS_IX_IS_SIGNER.0.iter(),
        SET_RPS_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetRpsIxData::new(rps).as_buf().into(),
    }
}

fn set_rps_ix_test_accs(keys: SetRpsIxKeysOwned, pool: PoolStateV2) -> AccountMap {
    const LAMPORTS: u64 = 1_000_000_000;

    let accs = NewSetRpsIxAccsBuilder::start()
        .with_pool_state(pool_state_v2_account(pool))
        .with_rps_auth(mock_sys_acc(LAMPORTS))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

fn set_rps_test(
    svm: &Mollusk,
    ix: Instruction,
    bef: &AccountMap,
    new_rps: u64,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let pool_pk = ix.accounts[SET_RPS_IX_ACCS_IDX_POOL_STATE].pubkey;
    let result = mollusk_exec(svm, &[ix], bef);

    match expected_err {
        None => {
            let aft: AccountMap = result.unwrap().resulting_accounts;

            let [pool_state_bef, pool_state_aft] = {
                acc_bef_aft(&pool_pk, bef, &aft).map(|acc| {
                    PoolStateV2Packed::of_acc_data(&acc.data)
                        .unwrap()
                        .into_pool_state_v2()
                })
            };

            let pool_state_bef_lookahead =
                pool_state_header_lookahead(pool_state_bef, svm.sysvars.clock.slot);

            assert_eq!(pool_state_aft.rps, new_rps);

            assert_diffs_pool_state_v2(
                &DiffsPoolStateV2 {
                    u64s: PoolStateV2U64s::default()
                        .with_withheld_lamports(Diff::Changed(
                            pool_state_bef.withheld_lamports,
                            pool_state_bef_lookahead.withheld_lamports,
                        ))
                        .with_protocol_fee_lamports(Diff::Changed(
                            pool_state_bef.protocol_fee_lamports,
                            pool_state_bef_lookahead.protocol_fee_lamports,
                        ))
                        .with_last_release_slot(Diff::Changed(
                            pool_state_bef.last_release_slot,
                            pool_state_bef_lookahead.last_release_slot,
                        )),
                    rps: Diff::Changed(
                        Rps::new(UQ0F63::new(pool_state_bef.rps).unwrap()).unwrap(),
                        Rps::new(UQ0F63::new(new_rps).unwrap()).unwrap(),
                    ),
                    ..Default::default()
                },
                &pool_state_bef,
                &pool_state_aft,
            );
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
        }
    }
}

#[test]
fn set_rps_correct_basic() {
    const NEW_RPS_RAW: u64 = *Rps::DEFAULT.as_inner().as_raw() + 1;

    // 69 + to avoid colliding with system prog
    let [rps_auth] = core::array::from_fn(|i| [69 + u8::try_from(i).unwrap(); 32]);

    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default().with_rps_authority(rps_auth),
        rps: Rps::DEFAULT,
        ..Default::default()
    }
    .into_pool_state_v2();

    let keys = NewSetRpsIxAccsBuilder::start()
        .with_pool_state(POOL_STATE_ID)
        .with_rps_auth(rps_auth)
        .build();

    SVM.with(|svm| {
        set_rps_test(
            svm,
            set_rps_ix(keys, NEW_RPS_RAW),
            &set_rps_ix_test_accs(keys, pool),
            NEW_RPS_RAW,
            Option::<ProgramError>::None,
        );
    });
}

fn args_ps_with_correct_keys(
    (new_rps_raw, ps): (u64, PoolStateV2),
) -> (SetRpsIxKeysOwned, u64, PoolStateV2) {
    (
        NewSetRpsIxAccsBuilder::start()
            .with_pool_state(POOL_STATE_ID)
            .with_rps_auth(ps.rps_authority)
            .build(),
        new_rps_raw,
        ps,
    )
}

fn to_inp(
    (keys, new_rps_raw, ps): (SetRpsIxKeysOwned, u64, PoolStateV2),
) -> (Instruction, AccountMap, u64) {
    (
        set_rps_ix(keys, new_rps_raw),
        set_rps_ix_test_accs(keys, ps),
        new_rps_raw,
    )
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap, u64)> {
    (
        any_rps_strat().prop_map(|r| *r.as_raw()),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            u64s: PoolStateV2U64s::default().with_last_release_slot(Some(Just(0).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn set_rps_correct_pt(
      (ix, bef, new_rps) in correct_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          set_rps_test(svm, ix, &bef, new_rps, Option::<ProgramError>::None);
      });
  }
}

fn invalid_rps_strat() -> impl Strategy<Value = (Instruction, AccountMap, u64)> {
    (
        prop_oneof![
            0..MIN_RPS_RAW,                         // below min
            (*UQ0F63::ONE.as_raw() + 1)..=u64::MAX, // above max
        ],
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            u64s: PoolStateV2U64s::default().with_last_release_slot(Some(Just(0).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn set_rps_invalid_rps_pt(
      (ix, bef, new_rps) in invalid_rps_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          set_rps_test(svm, ix, &bef, new_rps, Some(INVALID_INSTRUCTION_DATA));
      });
  }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap, u64)> {
    (
        any_rps_strat().prop_map(|r| *r.as_raw()),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            u64s: PoolStateV2U64s::default().with_last_release_slot(Some(Just(0).boxed())),
            ..Default::default()
        }),
    )
        .prop_flat_map(|(new_rps_raw, ps)| {
            (
                any_normal_pk()
                    .prop_filter("wrong rps authority", move |pk| *pk != ps.rps_authority),
                Just(new_rps_raw),
                Just(ps),
            )
        })
        .prop_map(|(wrong_rps_auth, new_rps_raw, ps)| {
            (
                NewSetRpsIxAccsBuilder::start()
                    .with_pool_state(POOL_STATE_ID)
                    .with_rps_auth(wrong_rps_auth)
                    .build(),
                new_rps_raw,
                ps,
            )
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn set_rps_unauthorized_pt(
      (ix, bef, new_rps) in unauthorized_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          set_rps_test(svm, ix, &bef, new_rps, Some(INVALID_ARGUMENT));
      });
  }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap, u64)> {
    correct_strat().prop_map(|(mut ix, bef, new_rps)| {
        ix.accounts[SET_RPS_IX_ACCS_IDX_RPS_AUTH].is_signer = false;
        (ix, bef, new_rps)
    })
}

proptest! {
  #[test]
  fn set_rps_missing_sig_pt(
      (ix, bef, new_rps) in missing_sig_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          set_rps_test(svm, ix, &bef, new_rps, Some(MISSING_REQUIRED_SIGNATURE));
      });
  }
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap, u64)> {
    (
        any_rps_strat().prop_map(|r| *r.as_raw()),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_disabled(Some(Just(true).boxed())),
            u64s: PoolStateV2U64s::default().with_last_release_slot(Some(Just(0).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn set_rps_disabled_pt(
      (ix, bef, new_rps) in disabled_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          set_rps_test(svm, ix, &bef, new_rps, Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled)));
      });
  }
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap, u64)> {
    (
        any_rps_strat().prop_map(|r| *r.as_raw()),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_rebalancing(Some(Just(true).boxed())),
            u64s: PoolStateV2U64s::default().with_last_release_slot(Some(Just(0).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn set_rps_rebalancing_pt(
      (ix, bef, new_rps) in rebalancing_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| {
          set_rps_test(svm, ix, &bef, new_rps, Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing)));
      });
  }
}

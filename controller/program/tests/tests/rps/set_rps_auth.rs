use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed},
    err::Inf1CtlErr,
    instructions::rps::set_rps_auth::{
        NewSetRpsAuthIxAccsBuilder, SetRpsAuthIxData, SetRpsAuthIxKeysOwned,
        SET_RPS_AUTH_IX_ACCS_IDX_NEW_RPS_AUTH, SET_RPS_AUTH_IX_ACCS_IDX_SIGNER,
        SET_RPS_AUTH_IX_IS_SIGNER, SET_RPS_AUTH_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state_v2, assert_diffs_pool_state_v2,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec,
    pool_state_v2_account, silence_mollusk_logs, AccountMap, Diff, DiffsPoolStateV2,
};
use jiminy_cpi::program_error::{ProgramError, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::Mollusk;
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_rps_auth_ix(keys: SetRpsAuthIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_RPS_AUTH_IX_IS_SIGNER.0.iter(),
        SET_RPS_AUTH_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetRpsAuthIxData::as_buf().into(),
    }
}

fn set_rps_auth_test_accs(keys: SetRpsAuthIxKeysOwned, pool: PoolStateV2) -> AccountMap {
    const LAMPORTS: u64 = 1_000_000_000;

    let accs = NewSetRpsAuthIxAccsBuilder::start()
        .with_pool_state(pool_state_v2_account(pool))
        .with_signer(mock_sys_acc(LAMPORTS))
        .with_new_rps_auth(mock_sys_acc(LAMPORTS))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

fn set_rps_auth_test(
    svm: &Mollusk,
    ix: Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let result = mollusk_exec(svm, std::slice::from_ref(&ix), bef);

    match expected_err {
        None => {
            let expected_new_rps_auth = ix.accounts[SET_RPS_AUTH_IX_ACCS_IDX_NEW_RPS_AUTH].pubkey;
            let aft = result.unwrap().resulting_accounts;

            let [pool_state_bef, pool_state_aft] = {
                acc_bef_aft(&POOL_STATE_ID.into(), bef, &aft).map(|acc| {
                    PoolStateV2Packed::of_acc_data(&acc.data)
                        .unwrap()
                        .into_pool_state_v2()
                })
            };

            assert_diffs_pool_state_v2(
                &DiffsPoolStateV2 {
                    addrs: PoolStateV2Addrs::default().with_rps_authority(Diff::Changed(
                        pool_state_bef.rps_authority,
                        expected_new_rps_auth.to_bytes(),
                    )),
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

fn set_rps_auth_correct_basic_test(signer_is_admin: bool) {
    // 69 + to avoid colliding with system prog
    let [curr_rps_auth, new_rps_auth, admin] =
        core::array::from_fn(|i| [69 + u8::try_from(i).unwrap(); 32]);

    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default()
            .with_rps_authority(curr_rps_auth)
            .with_admin(admin),
        ..Default::default()
    }
    .into_pool_state_v2();

    let signer = if signer_is_admin {
        admin
    } else {
        curr_rps_auth
    };

    let keys = NewSetRpsAuthIxAccsBuilder::start()
        .with_pool_state(POOL_STATE_ID)
        .with_signer(signer)
        .with_new_rps_auth(new_rps_auth)
        .build();
    SVM.with(|svm| {
        set_rps_auth_test(
            svm,
            set_rps_auth_ix(keys),
            &set_rps_auth_test_accs(keys, pool),
            Option::<ProgramError>::None,
        )
    });
}

#[test]
fn set_rps_auth_correct_basic_rps_auth_signer() {
    set_rps_auth_correct_basic_test(false); // rps_auth is signer
}

#[test]
fn set_rps_auth_correct_basic_admin_signer() {
    set_rps_auth_correct_basic_test(true); // admin is signer
}

fn to_inp((keys, ps): (SetRpsAuthIxKeysOwned, PoolStateV2)) -> (Instruction, AccountMap) {
    (set_rps_auth_ix(keys), set_rps_auth_test_accs(keys, ps))
}

fn correct_strat_params() -> impl Strategy<Value = ([u8; 32], PoolStateV2)> {
    (any_normal_pk(), any_pool_state_v2(Default::default()))
}

fn correct_strat_rps_auth() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat_params()
        .prop_map(|(new_rps_auth, ps)| {
            (
                NewSetRpsAuthIxAccsBuilder::start()
                    .with_pool_state(POOL_STATE_ID)
                    .with_signer(ps.rps_authority)
                    .with_new_rps_auth(new_rps_auth)
                    .build(),
                ps,
            )
        })
        .prop_map(to_inp)
}

fn correct_strat_admin() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat_params()
        .prop_map(|(new_rps_auth, ps)| {
            (
                NewSetRpsAuthIxAccsBuilder::start()
                    .with_pool_state(POOL_STATE_ID)
                    .with_signer(ps.admin)
                    .with_new_rps_auth(new_rps_auth)
                    .build(),
                ps,
            )
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn set_rps_auth_correct_rps_auth_pt(
      (ix, bef) in correct_strat_rps_auth(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| set_rps_auth_test(svm, ix, &bef, Option::<ProgramError>::None));
  }
}

proptest! {
  #[test]
  fn set_rps_auth_correct_admin_pt(
      (ix, bef) in correct_strat_admin(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| set_rps_auth_test(svm, ix, &bef, Option::<ProgramError>::None));
  }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat_params()
        .prop_flat_map(|(new_rps_auth, ps)| {
            (
                any_normal_pk().prop_filter("wrong signer", move |pk| {
                    *pk != ps.rps_authority && *pk != ps.admin
                }),
                Just(new_rps_auth),
                Just(ps),
            )
        })
        .prop_map(|(wrong_signer, new_rps_auth, ps)| {
            (
                NewSetRpsAuthIxAccsBuilder::start()
                    .with_pool_state(POOL_STATE_ID)
                    .with_signer(wrong_signer)
                    .with_new_rps_auth(new_rps_auth)
                    .build(),
                ps,
            )
        })
        .prop_map(to_inp)
}

proptest! {
  #[test]
  fn set_rps_auth_unauthorized_pt(
      (ix, bef) in unauthorized_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| set_rps_auth_test(svm, ix, &bef, Some(Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedSetRpsAuthoritySigner))));
  }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat_rps_auth().prop_map(|(mut ix, accs)| {
        ix.accounts[SET_RPS_AUTH_IX_ACCS_IDX_SIGNER].is_signer = false;
        (ix, accs)
    })
}

proptest! {
  #[test]
  fn set_rps_auth_missing_sig_pt(
      (ix, bef) in missing_sig_strat(),
  ) {
      silence_mollusk_logs();
      SVM.with(|svm| set_rps_auth_test(svm, ix, &bef, Some(MISSING_REQUIRED_SIGNATURE)));
  }
}

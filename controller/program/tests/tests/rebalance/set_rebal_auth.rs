use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed},
    err::Inf1CtlErr,
    instructions::rebalance::set_rebal_auth::{
        NewSetRebalAuthIxAccsBuilder, SetRebalAuthIxData, SetRebalAuthIxKeysOwned,
        SET_REBAL_AUTH_IX_ACCS_IDX_NEW, SET_REBAL_AUTH_IX_ACCS_IDX_SIGNER,
        SET_REBAL_AUTH_IX_IS_SIGNER, SET_REBAL_AUTH_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state_v2, assert_diffs_pool_state_v2,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec,
    pool_state_v2_account, pool_state_v2_u8_bools_normal_strat, silence_mollusk_logs, AccountMap,
    Diff, DiffsPoolStateV2, ExecResult, PoolStateV2FtaStrat,
};
use jiminy_cpi::program_error::{ProgramError, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::result::ProgramResult;
use proptest::{prelude::*, strategy::Union};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_rebal_auth_ix(keys: SetRebalAuthIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_REBAL_AUTH_IX_IS_SIGNER.0.iter(),
        SET_REBAL_AUTH_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetRebalAuthIxData::as_buf().into(),
    }
}

fn set_rebal_auth_test_accs(keys: SetRebalAuthIxKeysOwned, pool: PoolStateV2) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetRebalAuthIxAccsBuilder::start()
        .with_signer(mock_sys_acc(LAMPORTS))
        .with_new(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_v2_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `pool_state.rebalance_auth` at the end of ix
fn set_rebal_auth_test(
    ix: Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> [u8; 32] {
    let expected_new_rebal_auth = ix.accounts[SET_REBAL_AUTH_IX_ACCS_IDX_NEW].pubkey;
    let (aft, ExecResult { program_result, .. }) = SVM.with(|svm| mollusk_exec(svm, &[ix], bef));
    let [pool_state_bef, pool_state_aft] = acc_bef_aft(&POOL_STATE_ID.into(), bef, &aft).map(|a| {
        PoolStateV2Packed::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state_v2()
    });

    let old_rebal_auth = pool_state_bef.rebalance_authority;

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_diffs_pool_state_v2(
                &DiffsPoolStateV2 {
                    addrs: PoolStateV2Addrs::default().with_rebalance_authority(Diff::Changed(
                        old_rebal_auth,
                        expected_new_rebal_auth.to_bytes(),
                    )),
                    ..Default::default()
                },
                &pool_state_bef,
                &pool_state_aft,
            );
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }

    pool_state_aft.rebalance_authority
}

#[test]
fn admin_set_rebal_auth_test_correct_basic() {
    let [admin, new_rebal_auth] = core::array::from_fn(|i| [u8::try_from(i).unwrap(); 32]);
    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default().with_admin(admin),
        ..Default::default()
    }
    .into_pool_state_v2();
    let keys = NewSetRebalAuthIxAccsBuilder::start()
        .with_new(new_rebal_auth)
        .with_signer(admin)
        .with_pool_state(POOL_STATE_ID)
        .build();
    let ret = set_rebal_auth_test(
        set_rebal_auth_ix(keys),
        &set_rebal_auth_test_accs(keys, pool),
        Option::<ProgramError>::None,
    );
    assert_eq!(ret, new_rebal_auth);
}

/// generates (new_rebal_auth, pool_state)
fn correct_strat_params() -> impl Strategy<Value = ([u8; 32], PoolStateV2)> {
    (
        any_normal_pk(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
    )
}

fn admin_correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat_params()
        .prop_map(|(new_rebal_auth, ps)| {
            (
                NewSetRebalAuthIxAccsBuilder::start()
                    .with_new(new_rebal_auth)
                    .with_signer(ps.admin)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| (set_rebal_auth_ix(k), set_rebal_auth_test_accs(k, ps)))
}

fn rebal_auth_correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat_params()
        .prop_map(|(new_rebal_auth, ps)| {
            (
                NewSetRebalAuthIxAccsBuilder::start()
                    .with_new(new_rebal_auth)
                    .with_signer(ps.rebalance_authority)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| (set_rebal_auth_ix(k), set_rebal_auth_test_accs(k, ps)))
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat_params()
        .prop_flat_map(|(new_rebal_auth, ps)| {
            (
                any::<[u8; 32]>().prop_filter("", move |pk| {
                    *pk != ps.admin && *pk != ps.rebalance_authority
                }),
                Just(new_rebal_auth),
                Just(ps),
            )
        })
        .prop_map(|(unauthorized_signer, new_admin, ps)| {
            (
                NewSetRebalAuthIxAccsBuilder::start()
                    .with_new(new_admin)
                    .with_signer(unauthorized_signer)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| (set_rebal_auth_ix(k), set_rebal_auth_test_accs(k, ps)))
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    Union::new([
        admin_correct_strat().boxed(),
        rebal_auth_correct_strat().boxed(),
    ])
    .prop_map(|(mut ix, accs)| {
        ix.accounts[SET_REBAL_AUTH_IX_ACCS_IDX_SIGNER].is_signer = false;
        (ix, accs)
    })
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_disabled(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
        .prop_flat_map(|(new_rebal_auth, ps)| {
            (
                Union::new([ps.rebalance_authority, ps.admin].map(|signer| {
                    Just(
                        NewSetRebalAuthIxAccsBuilder::start()
                            .with_new(new_rebal_auth)
                            .with_signer(signer)
                            .with_pool_state(POOL_STATE_ID)
                            .build(),
                    )
                })),
                Just(ps),
            )
        })
        .prop_map(|(k, ps)| (set_rebal_auth_ix(k), set_rebal_auth_test_accs(k, ps)))
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_rebalancing(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
        .prop_flat_map(|(new_rebal_auth, ps)| {
            (
                Union::new([ps.rebalance_authority, ps.admin].map(|signer| {
                    Just(
                        NewSetRebalAuthIxAccsBuilder::start()
                            .with_new(new_rebal_auth)
                            .with_signer(signer)
                            .with_pool_state(POOL_STATE_ID)
                            .build(),
                    )
                })),
                Just(ps),
            )
        })
        .prop_map(|(k, ps)| (set_rebal_auth_ix(k), set_rebal_auth_test_accs(k, ps)))
}

proptest! {
    #[test]
    fn admin_set_rebal_auth_correct_pt(
        (ix, bef) in admin_correct_strat(),
    ) {
        silence_mollusk_logs();
        set_rebal_auth_test(ix, &bef, Option::<ProgramError>::None);
    }
}

proptest! {
    #[test]
    fn rebal_auth_set_rebal_auth_correct_pt(
        (ix, bef) in admin_correct_strat(),
    ) {
        silence_mollusk_logs();
        set_rebal_auth_test(ix, &bef, Option::<ProgramError>::None);
    }
}

proptest! {
    #[test]
    fn set_rebal_auth_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_rebal_auth_test(
            ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedSetRebalanceAuthoritySigner))
        );
    }
}

proptest! {
    #[test]
    fn set_rebal_auth_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_rebal_auth_test(ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

proptest! {
    #[test]
    fn set_rebal_auth_disabled_pt(
        (ix, bef) in disabled_strat(),
    ) {
        silence_mollusk_logs();
        set_rebal_auth_test(
            ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled))
        );
    }
}

proptest! {
    #[test]
    fn set_rebal_auth_rebalancing_pt(
        (ix, bef) in rebalancing_strat(),
    ) {
        silence_mollusk_logs();
        set_rebal_auth_test(
            ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing))
        );
    }
}

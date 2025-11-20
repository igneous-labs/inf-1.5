use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolState, PoolStatePacked},
    err::Inf1CtlErr,
    instructions::disable_pool::disable::{
        DisablePoolIxData, DisablePoolIxKeysOwned, NewDisablePoolIxAccsBuilder,
        DISABLE_POOL_IX_ACCS_IDX_SIGNER, DISABLE_POOL_IX_IS_SIGNER, DISABLE_POOL_IX_IS_WRITER,
    },
    keys::{DISABLE_POOL_AUTHORITY_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_disable_pool_auth_list, any_pool_state, assert_diffs_pool_state,
    assert_jiminy_prog_err, disable_pool_auth_list_account, gen_pool_state,
    keys_signer_writable_to_metas, list_sample_flat_map, mock_sys_acc, mollusk_exec,
    pool_state_account, silence_mollusk_logs, AccountMap, AnyPoolStateArgs, Diff,
    DiffsPoolStateArgs, GenPoolStateArgs, PoolStateBools, PoolStatePks,
};
use jiminy_cpi::program_error::{ProgramError, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::{prelude::*, strategy::Union};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{common::SVM, tests::disable_pool::common::MAX_DISABLE_POOL_AUTH_LIST_LEN};

fn disable_pool_ix(keys: DisablePoolIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        DISABLE_POOL_IX_IS_SIGNER.0.iter(),
        DISABLE_POOL_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: DisablePoolIxData::as_buf().into(),
    }
}

fn disable_pool_test_accs(
    keys: DisablePoolIxKeysOwned,
    pool: PoolState,
    // disable pool authority list
    dpal: Vec<[u8; 32]>,
) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewDisablePoolIxAccsBuilder::start()
        .with_signer(mock_sys_acc(LAMPORTS))
        .with_disable_pool_auth_list(disable_pool_auth_list_account(dpal))
        .with_pool_state(pool_state_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

fn disable_pool_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let (
        bef,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, ix, bef));

    let aft: AccountMap = resulting_accounts.into_iter().collect();
    let [pool_state_bef, pool_state_aft] =
        acc_bef_aft(&POOL_STATE_ID.into(), &bef, &aft).map(|a| {
            PoolStatePacked::of_acc_data(&a.data)
                .unwrap()
                .into_pool_state()
        });

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_diffs_pool_state(
                &DiffsPoolStateArgs {
                    bools: PoolStateBools::default()
                        // strict because ix is not supposed to succeed if pool already disabled
                        .with_is_disabled(Diff::StrictChanged(false, true)),
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
}

#[test]
fn disable_pool_test_correct_basic() {
    let admin = [69u8; 32];
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_admin(admin),
        ..Default::default()
    });
    let keys = NewDisablePoolIxAccsBuilder::start()
        .with_signer(admin)
        .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
        .build();
    disable_pool_test(
        &disable_pool_ix(keys),
        &disable_pool_test_accs(keys, pool, vec![]),
        Option::<ProgramError>::None,
    );
}

fn correct_keys(signer: [u8; 32]) -> DisablePoolIxKeysOwned {
    NewDisablePoolIxAccsBuilder::start()
        .with_signer(signer)
        .with_pool_state(POOL_STATE_ID)
        .with_disable_pool_auth_list(DISABLE_POOL_AUTHORITY_LIST_ID)
        .build()
}

fn to_inp(
    (k, ps, dpal): (DisablePoolIxKeysOwned, PoolState, Vec<[u8; 32]>),
) -> (Instruction, AccountMap) {
    (disable_pool_ix(k), disable_pool_test_accs(k, ps, dpal))
}

fn correct_admin_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        }),
        any_disable_pool_auth_list(0..=MAX_DISABLE_POOL_AUTH_LIST_LEN),
    )
        .prop_map(|(ps, dpal)| (correct_keys(ps.admin), ps, dpal))
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_pool_admin_correct_pt(
        (ix, bef) in correct_admin_strat(),
    ) {
        silence_mollusk_logs();
        disable_pool_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn correct_disable_auth_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_disable_pool_auth_list(1..=MAX_DISABLE_POOL_AUTH_LIST_LEN)
        .prop_flat_map(|l| {
            (
                list_sample_flat_map(l),
                any_pool_state(AnyPoolStateArgs {
                    bools: PoolStateBools::normal(),
                    ..Default::default()
                }),
            )
        })
        .prop_map(|((_, auth, dpal), ps)| (correct_keys(auth), ps, dpal))
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_pool_disable_auth_correct_pt(
        (ix, bef) in correct_disable_auth_strat(),
    ) {
        silence_mollusk_logs();
        disable_pool_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_disable_pool_auth_list(0..=MAX_DISABLE_POOL_AUTH_LIST_LEN),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        }),
    )
        .prop_flat_map(|(dpal, ps)| {
            let dpal_clone = dpal.clone();
            (
                any::<[u8; 32]>()
                    .prop_filter("", move |pk| *pk != ps.admin && !dpal_clone.contains(pk)),
                Just(dpal),
                Just(ps),
            )
        })
        .prop_map(|(unauth, dpal, ps)| (correct_keys(unauth), ps, dpal))
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_pool_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        disable_pool_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::UnauthorizedDisablePoolAuthoritySigner))
        );
    }
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    Union::new([
        correct_admin_strat().boxed(),
        correct_disable_auth_strat().boxed(),
    ])
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[DISABLE_POOL_IX_ACCS_IDX_SIGNER].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn disable_pool_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        disable_pool_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_disable_pool_auth_list(1..=MAX_DISABLE_POOL_AUTH_LIST_LEN)
        .prop_flat_map(|dpal| {
            (
                any_pool_state(AnyPoolStateArgs {
                    bools: PoolStateBools::normal().with_is_rebalancing(Some(Just(true).boxed())),
                    ..Default::default()
                }),
                list_sample_flat_map(dpal),
            )
        })
        .prop_flat_map(|(ps, (_, auth, dpal))| {
            (
                Union::new([Just(ps.admin), Just(auth)]),
                Just(ps),
                Just(dpal),
            )
        })
        .prop_map(|(auth, ps, dpal)| (correct_keys(auth), ps, dpal))
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_pool_rebalancing_pt(
        (ix, bef) in rebalancing_strat(),
    ) {
        silence_mollusk_logs();
        disable_pool_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing))
        );
    }
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_disable_pool_auth_list(1..=MAX_DISABLE_POOL_AUTH_LIST_LEN)
        .prop_flat_map(|dpal| {
            (
                any_pool_state(AnyPoolStateArgs {
                    bools: PoolStateBools::normal().with_is_disabled(Some(Just(true).boxed())),
                    ..Default::default()
                }),
                list_sample_flat_map(dpal),
            )
        })
        .prop_flat_map(|(ps, (_, auth, dpal))| {
            (
                Union::new([Just(ps.admin), Just(auth)]),
                Just(ps),
                Just(dpal),
            )
        })
        .prop_map(|(auth, ps, dpal)| (correct_keys(auth), ps, dpal))
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_pool_alrdy_disabled_pt(
        (ix, bef) in disabled_strat(),
    ) {
        silence_mollusk_logs();
        disable_pool_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled))
        );
    }
}

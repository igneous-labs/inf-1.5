use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolState, PoolStatePacked},
    err::Inf1CtlErr,
    instructions::disable_pool::enable::{
        EnablePoolIxData, EnablePoolIxKeysOwned, NewEnablePoolIxAccsBuilder,
        ENABLE_POOL_IX_ACCS_IDX_ADMIN, ENABLE_POOL_IX_IS_SIGNER, ENABLE_POOL_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_pool_state, assert_diffs_pool_state, assert_jiminy_prog_err, gen_pool_state,
    keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec, pool_state_account,
    silence_mollusk_logs, AccountMap, AnyPoolStateArgs, Diff, DiffsPoolStateArgs, GenPoolStateArgs,
    PoolStateBools, PoolStatePks,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn enable_pool_ix(keys: EnablePoolIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        ENABLE_POOL_IX_IS_SIGNER.0.iter(),
        ENABLE_POOL_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: EnablePoolIxData::as_buf().into(),
    }
}

fn enable_pool_test_accs(keys: EnablePoolIxKeysOwned, pool: PoolState) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewEnablePoolIxAccsBuilder::start()
        .with_admin(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

fn enable_pool_test(
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
                        // strict because ix is not supposed to succeed if pool already enabled
                        .with_is_disabled(Diff::StrictChanged(true, false)),
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
fn enable_pool_test_correct_basic() {
    let admin = [69u8; 32];
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_admin(admin),
        bools: PoolStateBools::default().with_is_disabled(true),
        ..Default::default()
    });
    let keys = NewEnablePoolIxAccsBuilder::start()
        .with_admin(admin)
        .with_pool_state(POOL_STATE_ID)
        .build();
    enable_pool_test(
        &enable_pool_ix(keys),
        &enable_pool_test_accs(keys, pool),
        Option::<ProgramError>::None,
    );
}

fn correct_keys(admin: [u8; 32]) -> EnablePoolIxKeysOwned {
    NewEnablePoolIxAccsBuilder::start()
        .with_admin(admin)
        .with_pool_state(POOL_STATE_ID)
        .build()
}

fn to_inp((k, ps): (EnablePoolIxKeysOwned, PoolState)) -> (Instruction, AccountMap) {
    (enable_pool_ix(k), enable_pool_test_accs(k, ps))
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal().with_is_disabled(Some(Just(true).boxed())),
        ..Default::default()
    })
    .prop_map(|ps| (correct_keys(ps.admin), ps))
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn enable_pool_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        enable_pool_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal().with_is_disabled(Some(Just(true).boxed())),
        ..Default::default()
    })
    .prop_flat_map(|ps| {
        (
            any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.admin),
            Just(ps),
        )
    })
    .prop_map(|(unauth, ps)| (correct_keys(unauth), ps))
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn enable_pool_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        enable_pool_test(
            &ix,
            &bef,
            Some(INVALID_ARGUMENT)
        );
    }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[ENABLE_POOL_IX_ACCS_IDX_ADMIN].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn enable_pool_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        enable_pool_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn enabled_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal().with_is_disabled(Some(Just(false).boxed())),
        ..Default::default()
    })
    .prop_map(|ps| (correct_keys(ps.admin), ps))
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn enable_pool_alrdy_enabled_pt(
        (ix, bef) in enabled_strat(),
    ) {
        silence_mollusk_logs();
        enable_pool_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolEnabled))
        );
    }
}

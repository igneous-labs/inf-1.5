use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed},
    err::Inf1CtlErr,
    instructions::admin::set_pricing_prog::{
        NewSetPricingProgIxAccsBuilder, SetPricingProgIxData, SetPricingProgIxKeysOwned,
        SET_PRICING_PROG_IX_ACCS_IDX_ADMIN, SET_PRICING_PROG_IX_ACCS_IDX_NEW,
        SET_PRICING_PROG_IX_IS_SIGNER, SET_PRICING_PROG_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state_v2, assert_diffs_pool_state_v2,
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_prog_acc, mock_sys_acc,
    mollusk_exec, pool_state_v2_account, pool_state_v2_u8_bools_normal_strat, silence_mollusk_logs,
    AccountMap, Diff, DiffsPoolStateV2, PoolStateV2FtaStrat,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_pricing_prog_ix(keys: SetPricingProgIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_PRICING_PROG_IX_IS_SIGNER.0.iter(),
        SET_PRICING_PROG_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetPricingProgIxData::as_buf().into(),
    }
}

fn set_pricing_prog_test_accs(keys: SetPricingProgIxKeysOwned, pool: PoolStateV2) -> AccountMap {
    // dont care, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetPricingProgIxAccsBuilder::start()
        .with_admin(mock_sys_acc(LAMPORTS))
        .with_new(mock_prog_acc(Default::default())) // dont care about programdata address
        .with_pool_state(pool_state_v2_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `pool_state.pricing_program` at the end of ix
fn set_pricing_prog_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> [u8; 32] {
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
            PoolStateV2Packed::of_acc_data(&a.data)
                .unwrap()
                .into_pool_state_v2()
        });

    let curr_pricing_prog = pool_state_bef.pricing_program;
    let expected_new_pricing_prog = ix.accounts[SET_PRICING_PROG_IX_ACCS_IDX_NEW].pubkey;

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_diffs_pool_state_v2(
                &DiffsPoolStateV2 {
                    addrs: PoolStateV2Addrs::default().with_pricing_program(Diff::Changed(
                        curr_pricing_prog,
                        expected_new_pricing_prog.to_bytes(),
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

    pool_state_aft.pricing_program
}

#[test]
fn set_pricing_prog_correct_basic() {
    let [admin, new_pp] = core::array::from_fn(|i| [u8::try_from(i).unwrap(); 32]);
    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default().with_admin(admin),
        ..Default::default()
    }
    .into_pool_state_v2();
    let keys = NewSetPricingProgIxAccsBuilder::start()
        .with_new(new_pp)
        .with_admin(admin)
        .with_pool_state(POOL_STATE_ID)
        .build();
    let ret = set_pricing_prog_test(
        &set_pricing_prog_ix(keys),
        &set_pricing_prog_test_accs(keys, pool),
        Option::<ProgramError>::None,
    );
    assert_eq!(ret, new_pp);
}

fn to_keys_and_accs(
    new_pp: impl Strategy<Value = [u8; 32]>,
    pool_state: impl Strategy<Value = PoolStateV2>,
) -> impl Strategy<Value = (Instruction, AccountMap)> {
    (new_pp, pool_state)
        .prop_map(|(new_pp, ps)| {
            (
                NewSetPricingProgIxAccsBuilder::start()
                    .with_new(new_pp)
                    .with_admin(ps.admin)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| (set_pricing_prog_ix(k), set_pricing_prog_test_accs(k, ps)))
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    to_keys_and_accs(
        any_normal_pk(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
    )
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        any_normal_pk(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
    )
        .prop_flat_map(|(new_pp, ps)| {
            (
                any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.admin),
                Just(new_pp),
                Just(ps),
            )
        })
        .prop_map(|(wrong_curr_admin, new_pp, ps)| {
            (
                NewSetPricingProgIxAccsBuilder::start()
                    .with_new(new_pp)
                    .with_admin(wrong_curr_admin)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| (set_pricing_prog_ix(k), set_pricing_prog_test_accs(k, ps)))
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[SET_PRICING_PROG_IX_ACCS_IDX_ADMIN].is_signer = false;
        (ix, accs)
    })
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    to_keys_and_accs(
        any_normal_pk(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_rebalancing(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    to_keys_and_accs(
        any_normal_pk(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_disabled(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
}

fn not_prog_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(ix, mut accs)| {
        accs.get_mut(&ix.accounts[SET_PRICING_PROG_IX_ACCS_IDX_NEW].pubkey)
            .unwrap()
            .executable = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn set_pricing_prog_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        set_pricing_prog_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

proptest! {
    #[test]
    fn set_pricing_prog_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_pricing_prog_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

proptest! {
    #[test]
    fn set_pricing_prog_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_pricing_prog_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

proptest! {
    #[test]
    fn set_pricing_prog_is_rebalancing_pt(
        (ix, bef) in rebalancing_strat(),
    ) {
        silence_mollusk_logs();
        set_pricing_prog_test(&ix, &bef, Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing)));
    }
}

proptest! {
    #[test]
    fn set_pricing_prog_is_disabled_pt(
        (ix, bef) in disabled_strat(),
    ) {
        silence_mollusk_logs();
        set_pricing_prog_test(&ix, &bef, Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled)));
    }
}

proptest! {
    #[test]
    fn set_pricing_prog_not_prog_pt(
        (ix, bef) in not_prog_strat(),
    ) {
        silence_mollusk_logs();
        set_pricing_prog_test(&ix, &bef, Some(Inf1CtlCustomProgErr(Inf1CtlErr::FaultyPricingProgram)));
    }
}

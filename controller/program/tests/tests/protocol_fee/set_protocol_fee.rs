use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed},
    err::Inf1CtlErr,
    instructions::protocol_fee::set_protocol_fee::{
        NewSetProtocolFeeIxAccsBuilder, SetProtocolFeeIxData, SetProtocolFeeIxKeysOwned,
        SET_PROTOCOL_FEE_IX_ACCS_IDX_ADMIN, SET_PROTOCOL_FEE_IX_IS_SIGNER,
        SET_PROTOCOL_FEE_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    typedefs::fee_nanos::{FeeNanos, MAX_FEE_NANOS},
    ID,
};
use inf1_test_utils::{
    any_ctl_fee_nanos_strat, any_pool_state_v2, assert_diffs_pool_state_v2, assert_jiminy_prog_err,
    keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec, pool_state_v2_account,
    pool_state_v2_u8_bools_normal_strat, silence_mollusk_logs, AccountMap, Diff, DiffsPoolStateV2,
    PoolStateV2FtaStrat,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_protocol_fee_ix(keys: SetProtocolFeeIxKeysOwned, protocol_fee_nanos: u32) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_PROTOCOL_FEE_IX_IS_SIGNER.0.iter(),
        SET_PROTOCOL_FEE_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetProtocolFeeIxData::new(protocol_fee_nanos)
            .as_buf()
            .into(),
    }
}

fn set_protocol_fee_ix_test_accs(keys: SetProtocolFeeIxKeysOwned, pool: PoolStateV2) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetProtocolFeeIxAccsBuilder::start()
        .with_admin(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_v2_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `pool_state` at the end of ix
fn set_protocol_fee_test(
    ix: Instruction,
    bef: &AccountMap,
    protocol_fee_nanos: u32,
    expected_err: Option<impl Into<ProgramError>>,
) -> PoolStateV2 {
    let result = SVM.with(|svm| mollusk_exec(svm, &[ix], bef));

    let pool_state_bef =
        PoolStateV2Packed::of_acc_data(&bef.get(&POOL_STATE_ID.into()).unwrap().data)
            .unwrap()
            .into_pool_state_v2();

    match expected_err {
        None => {
            let old_fee = FeeNanos::new(pool_state_bef.protocol_fee_nanos).unwrap();
            let new_fee = FeeNanos::new(protocol_fee_nanos).unwrap();
            let diffs = DiffsPoolStateV2 {
                protocol_fee_nanos: Diff::Changed(old_fee, new_fee),
                ..Default::default()
            };
            let resulting_accounts = result.unwrap().resulting_accounts;
            let pool_state_aft = PoolStateV2Packed::of_acc_data(
                &resulting_accounts.get(&POOL_STATE_ID.into()).unwrap().data,
            )
            .unwrap()
            .into_pool_state_v2();
            assert_diffs_pool_state_v2(&diffs, &pool_state_bef, &pool_state_aft);
            pool_state_aft
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
            pool_state_bef
        }
    }
}

#[test]
fn set_protocol_fee_test_correct_basic() {
    let [curr_fee_nanos, new_fee_nanos]: [u32; 2] =
        core::array::from_fn(|i| (i as u32 + 1) * 50_000_000);
    let admin = [69u8; 32];
    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default().with_admin(admin),
        protocol_fee_nanos: FeeNanos::new(curr_fee_nanos).unwrap(),
        ..Default::default()
    }
    .into_pool_state_v2();
    let keys = NewSetProtocolFeeIxAccsBuilder::start()
        .with_admin(admin)
        .with_pool_state(POOL_STATE_ID)
        .build();
    let ret = set_protocol_fee_test(
        set_protocol_fee_ix(keys, new_fee_nanos),
        &set_protocol_fee_ix_test_accs(keys, pool),
        new_fee_nanos,
        Option::<ProgramError>::None,
    );
    assert_eq!(ret.protocol_fee_nanos, new_fee_nanos);
}

fn correct_args_strat() -> impl Strategy<Value = u32> {
    any_ctl_fee_nanos_strat().prop_map(|fee| *fee)
}

fn invalid_args_strat() -> impl Strategy<Value = u32> {
    (MAX_FEE_NANOS + 1..).prop_map(|protocol_fee_nanos| protocol_fee_nanos)
}

fn args_ps_with_correct_keys(
    (protocol_fee_nanos, ps): (u32, PoolStateV2),
) -> (SetProtocolFeeIxKeysOwned, u32, PoolStateV2) {
    (
        NewSetProtocolFeeIxAccsBuilder::start()
            .with_admin(ps.admin)
            .with_pool_state(POOL_STATE_ID)
            .build(),
        protocol_fee_nanos,
        ps,
    )
}

fn to_test_inp(
    (k, protocol_fee_nanos, ps): (SetProtocolFeeIxKeysOwned, u32, PoolStateV2),
) -> (Instruction, AccountMap, u32) {
    (
        set_protocol_fee_ix(k, protocol_fee_nanos),
        set_protocol_fee_ix_test_accs(k, ps),
        protocol_fee_nanos,
    )
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap, u32)> {
    (
        correct_args_strat(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_correct_pt(
        (ix, bef, protocol_fee_nanos) in correct_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(ix, &bef, protocol_fee_nanos, Option::<ProgramError>::None);
    }
}

fn invalid_new_strat() -> impl Strategy<Value = (Instruction, AccountMap, u32)> {
    (
        invalid_args_strat(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat(),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_invalid_new_pt(
        (ix, bef, protocol_fee_nanos) in invalid_new_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(
            ix,
            &bef,
            protocol_fee_nanos,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::FeeTooHigh)),
        );
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap, u32)> {
    any_pool_state_v2(PoolStateV2FtaStrat {
        u8_bools: pool_state_v2_u8_bools_normal_strat(),
        ..Default::default()
    })
    .prop_flat_map(|ps| {
        (
            any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.admin),
            correct_args_strat(),
            Just(ps),
        )
    })
    .prop_map(|(wrong_admin, protocol_fee_nanos, ps)| {
        (
            NewSetProtocolFeeIxAccsBuilder::start()
                .with_admin(wrong_admin)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            protocol_fee_nanos,
            ps,
        )
    })
    .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_unauthorized_pt(
        (ix, bef, protocol_fee_nanos) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(ix, &bef, protocol_fee_nanos, Some(INVALID_ARGUMENT));
    }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap, u32)> {
    correct_strat().prop_map(|(mut ix, accs, protocol_fee_nanos)| {
        ix.accounts[SET_PROTOCOL_FEE_IX_ACCS_IDX_ADMIN].is_signer = false;
        (ix, accs, protocol_fee_nanos)
    })
}

proptest! {
    #[test]
    fn set_protocol_fee_missing_sig_pt(
        (ix, bef, protocol_fee_nanos) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(ix, &bef, protocol_fee_nanos, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap, u32)> {
    (
        correct_args_strat(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_disabled(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_pool_disabled_pt(
        (ix, bef, protocol_fee_nanos) in disabled_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(
            ix,
            &bef,
            protocol_fee_nanos,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled)),
        );
    }
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap, u32)> {
    (
        correct_args_strat(),
        any_pool_state_v2(PoolStateV2FtaStrat {
            u8_bools: pool_state_v2_u8_bools_normal_strat()
                .with_is_rebalancing(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_pool_rebalancing_pt(
        (ix, bef, protocol_fee_nanos) in rebalancing_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(
            ix,
            &bef,
            protocol_fee_nanos,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing)),
        );
    }
}

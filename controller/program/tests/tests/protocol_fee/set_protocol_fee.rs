use inf1_core::typedefs::fee_bps::BPS_DENOM;
use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolState, PoolStatePacked},
    err::Inf1CtlErr,
    instructions::protocol_fee::set_protocol_fee::{
        NewSetProtocolFeeIxAccsBuilder, SetProtocolFeeIxArgs, SetProtocolFeeIxData,
        SetProtocolFeeIxKeysOwned, SET_PROTOCOL_FEE_IX_ACCS_IDX_ADMIN,
        SET_PROTOCOL_FEE_IX_IS_SIGNER, SET_PROTOCOL_FEE_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_pool_state, assert_diffs_pool_state, assert_jiminy_prog_err, gen_pool_state,
    keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec, pool_state_account,
    silence_mollusk_logs, AccountMap, AnyPoolStateArgs, Diff, DiffsPoolStateArgs, GenPoolStateArgs,
    PoolStateBools, PoolStatePks, PoolStateU16s,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::{option, prelude::*, strategy::Union};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_protocol_fee_ix(keys: SetProtocolFeeIxKeysOwned, args: SetProtocolFeeIxArgs) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_PROTOCOL_FEE_IX_IS_SIGNER.0.iter(),
        SET_PROTOCOL_FEE_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetProtocolFeeIxData::new(args).as_buf().into(),
    }
}

fn set_protocol_fee_ix_test_accs(keys: SetProtocolFeeIxKeysOwned, pool: PoolState) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetProtocolFeeIxAccsBuilder::start()
        .with_admin(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `pool_state` at the end of ix
fn set_protocol_fee_test(
    ix: &Instruction,
    bef: &AccountMap,
    SetProtocolFeeIxArgs {
        trading_bps,
        lp_bps,
    }: SetProtocolFeeIxArgs,
    expected_err: Option<impl Into<ProgramError>>,
) -> PoolState {
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

    let diffs = [
        (
            pool_state_bef.trading_protocol_fee_bps,
            trading_bps,
            PoolStateU16s::with_trading_protocol_fee_bps
                as fn(PoolStateU16s<Diff<u16>>, Diff<u16>) -> PoolStateU16s<Diff<u16>>,
        ),
        (
            pool_state_bef.lp_protocol_fee_bps,
            lp_bps,
            PoolStateU16s::with_lp_protocol_fee_bps,
        ),
    ]
    .into_iter()
    .fold(
        DiffsPoolStateArgs::default(),
        |mut diffs, (old, new, with)| match new {
            None => diffs,
            Some(new) => {
                diffs.u16s = with(diffs.u16s, Diff::Changed(old, new));
                diffs
            }
        },
    );

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_diffs_pool_state(&diffs, &pool_state_bef, &pool_state_aft);
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }

    pool_state_aft
}

#[test]
fn set_protocol_fee_test_correct_basic() {
    let [curr_lp, new_lp, curr_trading, new_trading] =
        core::array::from_fn(|i| u16::from_le_bytes([i.try_into().unwrap(); 2]));
    let admin = [69u8; 32];
    let pool = gen_pool_state(GenPoolStateArgs {
        bools: PoolStateBools::default(),
        u16s: PoolStateU16s::default()
            .with_lp_protocol_fee_bps(curr_lp)
            .with_trading_protocol_fee_bps(curr_trading),
        pks: PoolStatePks::default().with_admin(admin),
        version: 1,
        ..Default::default()
    });
    let keys = NewSetProtocolFeeIxAccsBuilder::start()
        .with_admin(admin)
        .with_pool_state(POOL_STATE_ID)
        .build();
    let args = SetProtocolFeeIxArgs {
        trading_bps: Some(new_trading),
        lp_bps: Some(new_lp),
    };
    let ret = set_protocol_fee_test(
        &set_protocol_fee_ix(keys, args),
        &set_protocol_fee_ix_test_accs(keys, pool),
        args,
        Option::<ProgramError>::None,
    );
    assert_eq!(ret.lp_protocol_fee_bps, new_lp);
    assert_eq!(ret.trading_protocol_fee_bps, new_trading);
}

fn correct_args_strat() -> impl Strategy<Value = SetProtocolFeeIxArgs> {
    (0..=BPS_DENOM, 0..=BPS_DENOM)
        .prop_flat_map(|(t, l)| (option::of(Just(t)), option::of(Just(l))))
        .prop_map(|(trading_bps, lp_bps)| SetProtocolFeeIxArgs {
            trading_bps,
            lp_bps,
        })
}

fn invalid_args_strat() -> impl Strategy<Value = SetProtocolFeeIxArgs> {
    (BPS_DENOM + 1.., BPS_DENOM + 1..)
        .prop_flat_map(|(t, l)| {
            // at least one of the 2 must be some,
            // else its a valid arg
            Union::new([
                Just((Some(t), None)),
                Just((Some(t), Some(l))),
                Just((None, Some(l))),
            ])
        })
        .prop_map(|(trading_bps, lp_bps)| SetProtocolFeeIxArgs {
            trading_bps,
            lp_bps,
        })
}

fn args_ps_with_correct_keys(
    (args, ps): (SetProtocolFeeIxArgs, PoolState),
) -> (SetProtocolFeeIxKeysOwned, SetProtocolFeeIxArgs, PoolState) {
    (
        NewSetProtocolFeeIxAccsBuilder::start()
            .with_admin(ps.admin)
            .with_pool_state(POOL_STATE_ID)
            .build(),
        args,
        ps,
    )
}

fn to_test_inp(
    (k, args, ps): (SetProtocolFeeIxKeysOwned, SetProtocolFeeIxArgs, PoolState),
) -> (Instruction, AccountMap, SetProtocolFeeIxArgs) {
    (
        set_protocol_fee_ix(k, args),
        set_protocol_fee_ix_test_accs(k, ps),
        args,
    )
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap, SetProtocolFeeIxArgs)> {
    (
        correct_args_strat(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_correct_pt(
        (ix, bef, args) in correct_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(&ix, &bef, args, Option::<ProgramError>::None);
    }
}

fn invalid_new_strat() -> impl Strategy<Value = (Instruction, AccountMap, SetProtocolFeeIxArgs)> {
    (
        invalid_args_strat(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_invalid_new_pt(
        (ix, bef, args) in invalid_new_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(
            &ix,
            &bef,
            args,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::FeeTooHigh)),
        );
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap, SetProtocolFeeIxArgs)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal(),
        ..Default::default()
    })
    .prop_flat_map(|ps| {
        (
            any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.admin),
            correct_args_strat(),
            Just(ps),
        )
    })
    .prop_map(|(wrong_admin, args, ps)| {
        (
            NewSetProtocolFeeIxAccsBuilder::start()
                .with_admin(wrong_admin)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            args,
            ps,
        )
    })
    .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_unauthorized_pt(
        (ix, bef, args) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(&ix, &bef, args, Some(INVALID_ARGUMENT));
    }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap, SetProtocolFeeIxArgs)> {
    correct_strat().prop_map(|(mut ix, accs, args)| {
        ix.accounts[SET_PROTOCOL_FEE_IX_ACCS_IDX_ADMIN].is_signer = false;
        (ix, accs, args)
    })
}

proptest! {
    #[test]
    fn set_protocol_fee_missing_sig_pt(
        (ix, bef, args) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(&ix, &bef, args, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap, SetProtocolFeeIxArgs)> {
    (
        correct_args_strat(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal().with_is_disabled(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_pool_disabled_pt(
        (ix, bef, args) in disabled_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(
            &ix,
            &bef,
            args,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled)),
        );
    }
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap, SetProtocolFeeIxArgs)> {
    (
        correct_args_strat(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal().with_is_rebalancing(Some(Just(true).boxed())),
            ..Default::default()
        }),
    )
        .prop_map(args_ps_with_correct_keys)
        .prop_map(to_test_inp)
}

proptest! {
    #[test]
    fn set_protocol_fee_pool_rebalancing_pt(
        (ix, bef, args) in rebalancing_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_test(
            &ix,
            &bef,
            args,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing)),
        );
    }
}

use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed},
    instructions::protocol_fee::v1::set_protocol_fee_beneficiary::{
        NewSetProtocolFeeBeneficiaryIxAccsBuilder, SetProtocolFeeBeneficiaryIxData,
        SetProtocolFeeBeneficiaryIxKeysOwned, SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_CURR,
        SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_NEW, SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_SIGNER,
        SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    ID,
};
use inf1_test_utils::{
    any_normal_pk, any_pool_state_v2, assert_diffs_pool_state_v2, assert_jiminy_prog_err,
    keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec, pool_state_v2_account,
    silence_mollusk_logs, AccountMap, Diff, DiffsPoolStateV2,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_protocol_fee_beneficiary_ix(keys: SetProtocolFeeBeneficiaryIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_SIGNER.0.iter(),
        SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetProtocolFeeBeneficiaryIxData::as_buf().into(),
    }
}

fn set_protocol_fee_beneficiary_ix_test_accs(
    keys: SetProtocolFeeBeneficiaryIxKeysOwned,
    pool: PoolStateV2,
) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
        .with_curr(mock_sys_acc(LAMPORTS))
        .with_new(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_v2_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `pool_state.protocol_fee_beneficiary` at the end of ix
fn set_protocol_fee_beneficiary_test(
    ix: Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> [u8; 32] {
    let expected_new_ben = ix.accounts[SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_NEW].pubkey;
    let result = SVM.with(|svm| mollusk_exec(svm, &[ix], bef));

    let pool_state_bef =
        PoolStateV2Packed::of_acc_data(&bef.get(&POOL_STATE_ID.into()).unwrap().data)
            .unwrap()
            .into_pool_state_v2();
    let curr_ben = pool_state_bef.protocol_fee_beneficiary;

    match expected_err {
        None => {
            let resulting_accounts = result.unwrap().resulting_accounts;
            let pool_state_aft = PoolStateV2Packed::of_acc_data(
                &resulting_accounts.get(&POOL_STATE_ID.into()).unwrap().data,
            )
            .unwrap()
            .into_pool_state_v2();
            assert_diffs_pool_state_v2(
                &DiffsPoolStateV2 {
                    addrs: PoolStateV2Addrs::default().with_protocol_fee_beneficiary(
                        Diff::Changed(curr_ben, expected_new_ben.to_bytes()),
                    ),
                    ..Default::default()
                },
                &pool_state_bef,
                &pool_state_aft,
            );
            pool_state_aft.protocol_fee_beneficiary
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
            pool_state_bef.protocol_fee_beneficiary
        }
    }
}

#[test]
fn set_protocol_fee_beneficiary_test_correct_basic() {
    let [curr_ben, new_ben] = core::array::from_fn(|i| [u8::try_from(i).unwrap(); 32]);
    let pool = PoolStateV2FtaVals {
        addrs: PoolStateV2Addrs::default().with_protocol_fee_beneficiary(curr_ben),
        ..Default::default()
    }
    .into_pool_state_v2();
    let keys = NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
        .with_new(new_ben)
        .with_curr(curr_ben)
        .with_pool_state(POOL_STATE_ID)
        .build();
    let ret = set_protocol_fee_beneficiary_test(
        set_protocol_fee_beneficiary_ix(keys),
        &set_protocol_fee_beneficiary_ix_test_accs(keys, pool),
        Option::<ProgramError>::None,
    );
    assert_eq!(ret, new_ben);
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_pool_state_v2(Default::default()))
        .prop_map(|(new_ben, ps)| {
            (
                NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
                    .with_new(new_ben)
                    .with_curr(ps.protocol_fee_beneficiary)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| {
            (
                set_protocol_fee_beneficiary_ix(k),
                set_protocol_fee_beneficiary_ix_test_accs(k, ps),
            )
        })
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_pool_state_v2(Default::default()))
        .prop_flat_map(|(new_ben, ps)| {
            (
                any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.protocol_fee_beneficiary),
                Just(new_ben),
                Just(ps),
            )
        })
        .prop_map(|(wrong_curr_ben, new_ben, ps)| {
            (
                NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
                    .with_new(new_ben)
                    .with_curr(wrong_curr_ben)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| {
            (
                set_protocol_fee_beneficiary_ix(k),
                set_protocol_fee_beneficiary_ix_test_accs(k, ps),
            )
        })
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_CURR].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn set_protocol_fee_beneficiary_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_beneficiary_test(ix, &bef, Option::<ProgramError>::None);
    }
}

proptest! {
    #[test]
    fn set_protocol_fee_beneficiary_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_beneficiary_test(ix, &bef, Some(INVALID_ARGUMENT));
    }
}

proptest! {
    #[test]
    fn set_protocol_fee_beneficiary_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_beneficiary_test(ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

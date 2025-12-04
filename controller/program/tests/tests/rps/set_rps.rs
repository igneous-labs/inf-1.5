use inf1_ctl_jiminy::{
    accounts::pool_state::{
        PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals, PoolStateV2Packed, PoolStateV2U64s,
    },
    instructions::rps::set_rps::{
        NewSetRpsIxAccsBuilder, SetRpsIxData, SetRpsIxKeysOwned, SET_RPS_IX_ACCS_IDX_POOL_STATE,
        SET_RPS_IX_IS_SIGNER, SET_RPS_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    typedefs::{rps::Rps, uq0f63::UQ0F63},
    ID,
};
use inf1_svc_ag_core::calc::SvcCalcAg;
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_pool_state_v2, assert_jiminy_prog_err, keys_signer_writable_to_metas,
    mock_sys_acc, mollusk_exec, pool_state_v2_account, AccountMap, Diff, DiffsPoolStateV2,
};
use jiminy_cpi::program_error::ProgramError;
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

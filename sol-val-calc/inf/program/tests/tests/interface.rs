use std::ops::RangeInclusive;

use generic_array_struct::generic_array_struct;
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2Packed, svc::InfCalc, yields::release::ReleaseYieldParams,
};
use inf1_svc_generic::instructions::interface::{
    lst_to_sol::LST_TO_SOL_IX_DISCM, sol_to_lst::SOL_TO_LST_IX_DISCM, to_retdata, IxAccs, IxData,
    IxKeysOwned, IxPreAccs, IxPreAccsDestr, IxSufAccs, IxSufAccsDestr, IX_IS_SIGNER, IX_IS_WRITER,
};
use inf1_svc_inf_program::{CONST_KEYS_OWNED, CONST_PDAS, INF_MINT_ID, POOL_STATE_ID};
use inf1_test_utils::{
    assert_jiminy_prog_err, get_mint_supply, keys_signer_writable_to_metas, mock_gen_svc_state,
    mock_prog_acc, mock_progdata_acc, mollusk_exec, mollusk_with_clock_override, AccountMap,
    ClockArgs, ClockU64s, ClockU64sDestr, ProgramDataAddr,
};
use jiminy_entrypoint::program_error::ProgramError;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use expect_test::expect;
use inf1_test_utils::KeyedUiAccount;

use crate::common::SVM_MUT;

fn interface_keys() -> IxKeysOwned {
    IxAccs {
        pre: IxPreAccs::from_destr(IxPreAccsDestr {
            lst_mint: INF_MINT_ID,
        }),
        suf: IxSufAccs::from_destr(IxSufAccsDestr {
            state: CONST_PDAS.state().0,
            pool_state: POOL_STATE_ID,
            pool_prog: *CONST_KEYS_OWNED.pool_prog(),
            pool_progdata: CONST_PDAS.pool_progdata().0,
        }),
    }
}

fn interface_ix<const DISCM: u8>(keys: &IxKeysOwned, amt: u64) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(keys.seq(), IX_IS_SIGNER.seq(), IX_IS_WRITER.seq()),
        data: IxData::<DISCM>::new(amt).as_buf().into(),
    }
}

#[generic_array_struct(all pub)]
pub struct InterfaceTestAccs<T> {
    pub lst_mint: T,
    pub pool_state: T,
}

fn interface_test_accs(
    keys: &IxKeysOwned,
    last_upgrade_slot: u64,
    accs: InterfaceTestAccs<Account>,
) -> AccountMap {
    let InterfaceTestAccsDestr {
        lst_mint,
        pool_state,
    } = accs.into_destr();

    keys.seq()
        .copied()
        .map(Into::into)
        .zip(
            IxAccs {
                pre: IxPreAccs::from_destr(IxPreAccsDestr { lst_mint }),
                suf: IxSufAccs::from_destr(IxSufAccsDestr {
                    state: mock_gen_svc_state(
                        inf1_svc_generic::accounts::state::State {
                            manager: *CONST_KEYS_OWNED.init_manager(),
                            last_upgrade_slot,
                        },
                        Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
                    ),
                    pool_state,
                    pool_prog: mock_prog_acc(ProgramDataAddr::Raw(Default::default())),
                    pool_progdata: mock_progdata_acc(last_upgrade_slot),
                }),
            }
            .seq()
            .cloned(),
        )
        .collect()
}

/// Executes the interface instruction at the given clock slot.
///
/// On success, reads `pool_state` + `inf_mint` from `bef`, computes `InfCalc`
/// with lookahead at `slot`, and asserts the program's return data matches,
/// and returns Some(range parsed from return data)
///
/// On error, asserts the expected [`ProgramError`] and returns None
fn interface_test(
    ix: &Instruction,
    bef: &AccountMap,
    slot: u64,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<RangeInclusive<u64>> {
    SVM_MUT.with(|svm| {
        mollusk_with_clock_override(
            &mut svm.borrow_mut(),
            &ClockArgs {
                u64s: ClockU64s::from_destr(ClockU64sDestr {
                    slot: Some(slot),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |svm| {
                let result = mollusk_exec(svm, core::slice::from_ref(ix), bef);

                match expected_err {
                    None => {
                        let ok = result.unwrap();
                        let amt = IxData::<0>::parse_no_discm(ix.data.last_chunk().unwrap());
                        let discm = ix.data[0];

                        let pool_state_packed = PoolStateV2Packed::of_acc_data(
                            &bef.get(&Pubkey::new_from_array(POOL_STATE_ID))
                                .unwrap()
                                .data,
                        )
                        .unwrap();
                        let pool_state = pool_state_packed.into_pool_state_v2();
                        let supply = get_mint_supply(
                            &bef.get(&Pubkey::new_from_array(INF_MINT_ID)).unwrap().data,
                        );

                        let params = ReleaseYieldParams::new(&pool_state, slot).unwrap();
                        let calc = InfCalc::new(&pool_state, supply).lookahead(params).unwrap();

                        let result = if discm == LST_TO_SOL_IX_DISCM {
                            calc.svc_lst_to_sol(amt).unwrap()
                        } else {
                            calc.svc_sol_to_lst(amt).unwrap()
                        };
                        let expected = to_retdata(&result);

                        assert_eq!(ok.return_data, expected.to_vec());

                        Some(result)
                    }
                    Some(e) => {
                        assert_jiminy_prog_err(&result.unwrap_err(), e);
                        None
                    }
                }
            },
        )
    })
}

#[test]
fn lst_to_sol_fixture_snapshot() {
    let slot = 0u64;
    let (_, fixture_pool_state) =
        KeyedUiAccount::from_test_fixtures_json("pool-state").into_keyed_account();
    let (_, fixture_inf_mint) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint").into_keyed_account();

    let keys = interface_keys();
    let am = interface_test_accs(
        &keys,
        slot,
        InterfaceTestAccs::from_destr(InterfaceTestAccsDestr {
            lst_mint: fixture_inf_mint,
            pool_state: fixture_pool_state,
        }),
    );
    let ix = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, 1_000_000_000);

    let result = interface_test(&ix, &am, slot, Option::<ProgramError>::None).unwrap();

    expect![[r#"
        2228787865..=2228787865
    "#]]
    .assert_debug_eq(&result);
}

#[test]
fn sol_to_lst_fixture_snapshot() {
    let slot = 0u64;
    let (_, fixture_pool_state) =
        KeyedUiAccount::from_test_fixtures_json("pool-state").into_keyed_account();
    let (_, fixture_inf_mint) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint").into_keyed_account();

    let keys = interface_keys();
    let am = interface_test_accs(
        &keys,
        slot,
        InterfaceTestAccs::from_destr(InterfaceTestAccsDestr {
            lst_mint: fixture_inf_mint,
            pool_state: fixture_pool_state,
        }),
    );
    let ix = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, 2_228_787_865);

    let result = interface_test(&ix, &am, slot, Option::<ProgramError>::None).unwrap();

    expect![[r#"
        1000000000..=1000000000
    "#]]
    .assert_debug_eq(&result);
}

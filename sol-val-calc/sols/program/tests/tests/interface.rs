use core::ops::RangeInclusive;

use generic_array_struct::generic_array_struct;
use inf1_svc_generic::{
    accounts::state::State,
    instructions::interface::{
        lst_to_sol::LST_TO_SOL_IX_DISCM, sol_to_lst::SOL_TO_LST_IX_DISCM, to_retdata, IxAccs,
        IxData, IxKeysOwned, IxPreAccs, IxPreAccsDestr, IxSufAccs, IxSufAccsDestr, IX_IS_SIGNER,
        IX_IS_WRITER, IX_PRE_ACCS_IDX_LST_MINT, IX_SUF_ACCS_IDX_POOL_STATE,
    },
};
use inf1_svc_sols_core::{
    calc::SolsCalc,
    keys::{CONST_KEYS_OWNED, CONST_PDAS},
    sanctum_sols_core::{
        accounts::{PoolV1Acc, PoolV1AddrVals, PoolV1AddrsDestr, PoolV1PdaBumps, PoolV1PdasDestr},
        typedefs::PoolFlags,
    },
};
use inf1_test_utils::{
    any_normal_pk, assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_gen_svc_state,
    mock_mint, mock_prog_acc, mock_progdata_acc, mollusk_exec, perturb_key_arr_flat_map_gen,
    raw_mint, silence_mollusk_logs, AccountMap, ProgramDataAddr,
};
use jiminy_entrypoint::program_error::{ProgramError, INVALID_ACCOUNT_DATA, INVALID_ARGUMENT};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use expect_test::expect;
use proptest::prelude::*;

use crate::common::SVM_MUT;

/// arbitrary non-sysvar mint for the snapshot tests
const LST_MINT: [u8; 32] = [3u8; 32];

fn find_pool_pda(mint: &[u8; 32]) -> ([u8; 32], u8) {
    let (pda, bump) = Pubkey::find_program_address(
        &[mint],
        &Pubkey::new_from_array(*CONST_KEYS_OWNED.pool_prog()),
    );
    (pda.to_bytes(), bump)
}

fn interface_keys_for(lst_mint: [u8; 32]) -> IxKeysOwned {
    let (pool_state, _) = find_pool_pda(&lst_mint);
    IxAccs {
        pre: IxPreAccs::from_destr(IxPreAccsDestr { lst_mint }),
        suf: IxSufAccs::from_destr(IxSufAccsDestr {
            state: CONST_PDAS.state().0,
            pool_state,
            pool_prog: *CONST_KEYS_OWNED.pool_prog(),
            pool_progdata: CONST_PDAS.pool_progdata().0,
        }),
    }
}

fn interface_keys() -> IxKeysOwned {
    interface_keys_for(LST_MINT)
}

fn interface_ix<const DISCM: u8>(keys: &IxKeysOwned, amt: u64) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(keys.seq(), IX_IS_SIGNER.seq(), IX_IS_WRITER.seq()),
        data: IxData::<DISCM>::new(amt).as_buf().into(),
    }
}

fn pool_v1_acc(mint: [u8; 32]) -> PoolV1Acc {
    let (_, pool_bump) = find_pool_pda(&mint);
    PoolV1Acc {
        addrs: PoolV1AddrVals::from_destr(PoolV1AddrsDestr {
            mint,
            // dont-cares
            ..Default::default()
        }),
        // dont-cares, SolValCalc impl always returns 1:1
        lamports: Default::default(),
        // dont-cares
        nanos: Default::default(),
        bumps: PoolV1PdaBumps::from_destr(PoolV1PdasDestr {
            pool: pool_bump,
            // dont-cares
            ..Default::default()
        }),
        flags: PoolFlags::V1.into(),
    }
}

fn mock_sols_pool_raw(data: Vec<u8>) -> Account {
    Account {
        // dont-cares, svc always returns 1:1
        lamports: 1_000_000_000,
        data,
        owner: Pubkey::new_from_array(*CONST_KEYS_OWNED.pool_prog()),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

fn mock_sols_pool(mint: [u8; 32]) -> Account {
    mock_sols_pool_raw(pool_v1_acc(mint).as_acc_data_arr().into())
}

#[generic_array_struct(all pub)]
#[derive(Debug, Clone, Copy)]
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
                        State {
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

/// Executes the interface instruction.
///
/// On success, computes the expected `SolsCalc` result, asserts the program's
/// return data matches, and returns Some(range parsed from return data).
///
/// On error, asserts the expected [`ProgramError`] and returns None
fn interface_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<RangeInclusive<u64>> {
    SVM_MUT.with(|svm| {
        let result = mollusk_exec(&svm.borrow(), core::slice::from_ref(ix), bef);

        match expected_err {
            None => {
                let ok = result.unwrap();
                let amt = IxData::<0>::parse_no_discm(ix.data.last_chunk().unwrap());
                let discm = ix.data[0];

                let result = if discm == LST_TO_SOL_IX_DISCM {
                    SolsCalc.svc_lst_to_sol(amt).unwrap()
                } else {
                    SolsCalc.svc_sol_to_lst(amt).unwrap()
                };
                let expected = to_retdata(&result);

                assert_eq!(ok.return_data, expected);

                Some(result)
            }
            Some(e) => {
                assert_jiminy_prog_err(&result.unwrap_err(), e);
                None
            }
        }
    })
}

#[test]
fn lst_to_sol_snapshot() {
    let slot = 0u64;
    let keys = interface_keys();
    let am = interface_test_accs(
        &keys,
        slot,
        InterfaceTestAccs::from_destr(InterfaceTestAccsDestr {
            lst_mint: mock_mint(raw_mint(None, None, 1_000_000_000, 9)),
            pool_state: mock_sols_pool(LST_MINT),
        }),
    );
    let ix = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, 1_000_000_000);

    let result = interface_test(&ix, &am, Option::<ProgramError>::None).unwrap();

    expect![[r#"
        1000000000..=1000000000
    "#]]
    .assert_debug_eq(&result);
}

#[test]
fn sol_to_lst_snapshot() {
    let slot = 0u64;
    let keys = interface_keys();
    let am = interface_test_accs(
        &keys,
        slot,
        InterfaceTestAccs::from_destr(InterfaceTestAccsDestr {
            lst_mint: mock_mint(raw_mint(None, None, 1_000_000_000, 9)),
            pool_state: mock_sols_pool(LST_MINT),
        }),
    );
    let ix = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, 1_000_000_000);

    let result = interface_test(&ix, &am, Option::<ProgramError>::None).unwrap();

    expect![[r#"
        1000000000..=1000000000
    "#]]
    .assert_debug_eq(&result);
}

#[generic_array_struct(all pub)]
#[derive(Debug, Clone, Copy)]
struct InterfaceTestU64s<T> {
    pub amt: T,
    pub slot: T,
}
type InterfaceTestU64Vals = InterfaceTestU64s<u64>;

fn correct_strat() -> impl Strategy<
    Value = (
        IxKeysOwned,
        InterfaceTestU64Vals,
        InterfaceTestAccs<Account>,
    ),
> {
    (any_normal_pk(), 1u64..=1_000_000_000_000, 0u64..=1_000_000).prop_map(
        |(lst_mint, amt, slot)| {
            (
                interface_keys_for(lst_mint),
                InterfaceTestU64s::from_destr(InterfaceTestU64sDestr { amt, slot }),
                InterfaceTestAccs::from_destr(InterfaceTestAccsDestr {
                    lst_mint: mock_mint(raw_mint(None, None, 0, 0)),
                    pool_state: mock_sols_pool(lst_mint),
                }),
            )
        },
    )
}

proptest! {
    #[test]
    fn correct_pt((keys, u64s, accs) in correct_strat()) {
        silence_mollusk_logs();
        let am = interface_test_accs(&keys, *u64s.slot(), accs);
        let ix_l2s = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, *u64s.amt());
        let ix_s2l = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, *u64s.amt());
        for ix in [ix_l2s, ix_s2l] {
            interface_test(&ix, &am, Option::<ProgramError>::None).unwrap();
        }
    }
}

/// `(keys, last_upgrade_slot, pool_v1_account)` to input into [`interface_test_accs`]
/// after transformation
type StratVal = (IxKeysOwned, u64, Account);

fn invalid_pool_vers_strat() -> impl Strategy<Value = StratVal> {
    any_normal_pk().prop_map(|mint| {
        let mut pool = pool_v1_acc(mint);
        // 0 is not a valid vers bitpattern
        pool.flags = 0;
        (
            interface_keys_for(mint),
            0,
            mock_sols_pool_raw(pool.as_acc_data_arr().into()),
        )
    })
}

proptest! {
    #[test]
    fn invalid_pool_vers_pt((keys, slot, pool_state) in invalid_pool_vers_strat()) {
        silence_mollusk_logs();
        let am = interface_test_accs(
            &keys,
            slot,
            InterfaceTestAccs::from_destr(InterfaceTestAccsDestr {
                lst_mint: mock_mint(raw_mint(None, None, 0, 0)),
                pool_state,
            }),
        );
        let ix_l2s = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, 1_000_000);
        let ix_s2l = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, 1_000_000);
        for ix in [ix_l2s, ix_s2l] {
            prop_assert!(interface_test(&ix, &am, Some(INVALID_ACCOUNT_DATA)).is_none());
        }
    }
}

fn truncated_pool_data_strat() -> impl Strategy<Value = StratVal> {
    any_normal_pk().prop_map(|mint| {
        let mut data: Vec<u8> = pool_v1_acc(mint).as_acc_data_arr().into();
        data.pop();
        (interface_keys_for(mint), 0, mock_sols_pool_raw(data))
    })
}

proptest! {
    #[test]
    fn truncated_pool_data_pt((keys, slot, pool_state) in truncated_pool_data_strat()) {
        silence_mollusk_logs();
        let am = interface_test_accs(
            &keys,
            slot,
            InterfaceTestAccs::from_destr(InterfaceTestAccsDestr {
                lst_mint: mock_mint(raw_mint(None, None, 0, 0)),
                pool_state,
            }),
        );
        let ix_l2s = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, 1_000_000);
        let ix_s2l = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, 1_000_000);
        for ix in [ix_l2s, ix_s2l] {
            prop_assert!(interface_test(&ix, &am, Some(INVALID_ACCOUNT_DATA)).is_none());
        }
    }
}

fn perturb_mint_id_strat() -> impl Strategy<
    Value = (
        IxKeysOwned,
        InterfaceTestU64Vals,
        InterfaceTestAccs<Account>,
    ),
> {
    correct_strat().prop_flat_map(move |(IxAccs { pre, suf }, u64s, accs)| {
        (
            perturb_key_arr_flat_map_gen(IX_PRE_ACCS_IDX_LST_MINT)(pre.0).prop_map(move |pre| {
                IxAccs {
                    pre: IxPreAccs(pre),
                    suf,
                }
            }),
            Just(u64s),
            Just(accs),
        )
    })
}

proptest! {
    #[test]
    fn wrong_mint_pt((keys, u64s, accs) in perturb_mint_id_strat()) {
        silence_mollusk_logs();
        let am = interface_test_accs(&keys, *u64s.slot(), accs);
        let ix_l2s = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, *u64s.amt());
        let ix_s2l = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, *u64s.amt());
        for ix in [ix_l2s, ix_s2l] {
            prop_assert!(interface_test(&ix, &am, Some(INVALID_ARGUMENT)).is_none());
        }
    }
}

fn perturb_pool_state_id_strat() -> impl Strategy<
    Value = (
        IxKeysOwned,
        InterfaceTestU64Vals,
        InterfaceTestAccs<Account>,
    ),
> {
    correct_strat().prop_flat_map(move |(IxAccs { pre, suf }, u64s, accs)| {
        (
            perturb_key_arr_flat_map_gen(IX_SUF_ACCS_IDX_POOL_STATE)(suf.0).prop_map(move |suf| {
                IxAccs {
                    pre,
                    suf: IxSufAccs(suf),
                }
            }),
            Just(u64s),
            Just(accs),
        )
    })
}

proptest! {
    #[test]
    fn wrong_pool_state_pt((keys, u64s, accs) in perturb_pool_state_id_strat()) {
        silence_mollusk_logs();
        let am = interface_test_accs(&keys, *u64s.slot(), accs);
        let ix_l2s = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, *u64s.amt());
        let ix_s2l = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, *u64s.amt());
        for ix in [ix_l2s, ix_s2l] {
            prop_assert!(interface_test(&ix, &am, Some(INVALID_ARGUMENT)).is_none());
        }
    }
}

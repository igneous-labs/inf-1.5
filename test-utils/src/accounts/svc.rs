use inf1_ctl_core::svc::InfDummyCalcAccs;
use inf1_svc_ag_core::{
    inf1_svc_core::traits::SolValCalcAccs,
    inf1_svc_generic::{
        accounts::state::StatePacked,
        instructions::{IxSufAccs, NewIxSufAccsBuilder},
    },
    instructions::SvcCalcAccsAg,
    SvcAg,
};
use inf1_svc_marinade_core::instructions::sol_val_calc::MarinadeCalcAccs;
use inf1_svc_spl_core::{
    inf1_svc_generic::accounts::state::State,
    instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
};
use inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs;
use sanctum_spl_stake_pool_core::StakePool;
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::{mock_prog_acc, mock_progdata_acc, AccountMap, KeyedUiAccount, ProgramDataAddr};

/// Owner should be 1 of the 3 stake pool programs
pub fn mock_spl_stake_pool(a: &StakePool, owner: Pubkey) -> Account {
    let mut data = Vec::new();
    a.borsh_ser(&mut data).unwrap();
    Account {
        lamports: 5_143_440, // solana rent 611
        data,
        owner,
        executable: false,
        rent_epoch: u64::MAX,
    }
}

pub fn mock_marinade_state(a: &sanctum_marinade_liquid_staking_core::State) -> Account {
    let mut data = Vec::new();
    a.borsh_ser(&mut data).unwrap();
    Account {
        lamports: u32::MAX.into(), // more than enough for rent
        data,
        owner: sanctum_marinade_liquid_staking_core::MARINADE_STAKING_PROGRAM.into(),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

pub fn mock_gpc_state(a: &State, svc_prog: Pubkey) -> Account {
    Account {
        lamports: 1_169_280, // solana rent 40
        data: a.as_acc_data_arr().into(),
        owner: svc_prog,
        executable: false,
        rent_epoch: u64::MAX,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GpcAccParams<T> {
    pub pool: T,
    pub gpc_state: State,
    pub last_prog_upg_slot: u64,
}

pub type MarinadeSvcAccParams = GpcAccParams<sanctum_marinade_liquid_staking_core::State>;
pub type SplSvcAccParams = GpcAccParams<StakePool>;

pub type SvcAccParamsAg = SvcAg<
    (),
    (),
    (MarinadeCalcAccs, MarinadeSvcAccParams),
    (SanctumSplCalcAccs, SplSvcAccParams),
    (SanctumSplMultiCalcAccs, SplSvcAccParams),
    (SplCalcAccs, SplSvcAccParams),
    WsolCalcAccs,
>;

pub fn msol_fixture_svc_suf_accs() -> (MarinadeCalcAccs, AccountMap) {
    let [(_, marinade_state), (_, gpc_state)] = ["msol-pool", "marinade-calc-state"]
        .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account());
    let keys = IxSufAccs(MarinadeCalcAccs.suf_keys_owned().0.map(Pubkey::from));
    (
        MarinadeCalcAccs,
        generic_svc_accs(
            keys,
            StatePacked::of_acc_data(&gpc_state.data)
                .unwrap()
                .into_state()
                .last_upgrade_slot,
            GpcAccs {
                gpc_state,
                pool_state: marinade_state,
            },
        ),
    )
}

pub fn jupsol_fixture_svc_suf_accs() -> (SanctumSplMultiCalcAccs, AccountMap) {
    let [(jupsol_pool_addr, stake_pool), (_, gpc_state)] =
        ["jupsol-pool", "sanctum-spl-multi-calc-state"]
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account());
    let calc_accs = SanctumSplMultiCalcAccs {
        stake_pool_addr: jupsol_pool_addr.to_bytes(),
    };
    let keys = IxSufAccs(calc_accs.suf_keys_owned().0.map(Pubkey::from));
    (
        calc_accs,
        generic_svc_accs(
            keys,
            StatePacked::of_acc_data(&gpc_state.data)
                .unwrap()
                .into_state()
                .last_upgrade_slot,
            GpcAccs {
                gpc_state,
                pool_state: stake_pool,
            },
        ),
    )
}

pub fn svc_accs(params: SvcAccParamsAg) -> (SvcCalcAccsAg, AccountMap) {
    let (calc_accs, keys, last_prog_upg_slot, gpc_accs) = match &params {
        SvcAg::Lido(_) => todo!(),
        SvcAg::Inf(_) => return (SvcCalcAccsAg::Inf(InfDummyCalcAccs), Default::default()),
        SvcAg::Wsol(a) => return (SvcAg::Wsol(*a), Default::default()),
        SvcAg::Marinade((
            calc_accs,
            GpcAccParams {
                pool,
                gpc_state,
                last_prog_upg_slot,
            },
        )) => (
            SvcCalcAccsAg::Marinade(*calc_accs),
            calc_accs.suf_keys_owned(),
            last_prog_upg_slot,
            GpcAccs {
                gpc_state: mock_gpc_state(gpc_state, Pubkey::from(*params.svc_program_id())),
                pool_state: mock_marinade_state(pool),
            },
        ),
        SvcAg::SanctumSpl((_, p)) | SvcAg::SanctumSplMulti((_, p)) | SvcAg::Spl((_, p)) => {
            let (calc_accs, keys) = match params {
                SvcAg::SanctumSpl((a, _)) => (SvcCalcAccsAg::SanctumSpl(a), a.suf_keys_owned()),
                SvcAg::SanctumSplMulti((a, _)) => {
                    (SvcCalcAccsAg::SanctumSplMulti(a), a.suf_keys_owned())
                }
                SvcAg::Spl((a, _)) => (SvcCalcAccsAg::Spl(a), a.suf_keys_owned()),
                _ => unreachable!(),
            };
            let SplSvcAccParams {
                pool,
                gpc_state,
                last_prog_upg_slot,
            } = p;
            (
                calc_accs,
                keys,
                last_prog_upg_slot,
                GpcAccs {
                    gpc_state: mock_gpc_state(gpc_state, Pubkey::from(*params.svc_program_id())),
                    pool_state: mock_spl_stake_pool(pool, Pubkey::from(*keys.pool_prog())),
                },
            )
        }
    };
    (
        calc_accs,
        generic_svc_accs(
            IxSufAccs(keys.0.map(Pubkey::from)),
            *last_prog_upg_slot,
            gpc_accs,
        ),
    )
}

struct GpcAccs {
    gpc_state: Account,
    pool_state: Account,
}

fn generic_svc_accs(
    keys: IxSufAccs<Pubkey>,
    last_prog_upg_slot: u64,
    GpcAccs {
        gpc_state,
        pool_state,
    }: GpcAccs,
) -> AccountMap {
    let accs = NewIxSufAccsBuilder::start()
        .with_pool_prog(mock_prog_acc(ProgramDataAddr::Raw(*keys.pool_progdata())))
        .with_pool_progdata(mock_progdata_acc(last_prog_upg_slot))
        .with_state(gpc_state)
        .with_pool_state(pool_state)
        .build();
    keys.0.into_iter().zip(accs.0).collect()
}

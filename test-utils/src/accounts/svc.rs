use inf1_ctl_core::svc::InfDummyCalcAccs;
use inf1_svc_ag_core::{
    inf1_svc_core::traits::SolValCalcAccs,
    inf1_svc_generic::{accounts::state::StatePacked, instructions::IxSufAccs},
    instructions::SvcCalcAccsAg,
    SvcAg,
};
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

pub fn mock_gpc_state(a: &State, svc_prog: Pubkey) -> Account {
    Account {
        lamports: 1_169_280, // solana rent 40
        data: a.as_acc_data_arr().into(),
        owner: svc_prog,
        executable: false,
        rent_epoch: u64::MAX,
    }
}

#[derive(Debug, Clone)]
pub struct SplSvcAccParams {
    pub pool: StakePool,
    pub gpc_state: State,
    pub last_prog_upg_slot: u64,
}

pub type SvcAccParamsAg = SvcAg<
    (),
    (),
    (),
    (SanctumSplCalcAccs, SplSvcAccParams),
    (SanctumSplMultiCalcAccs, SplSvcAccParams),
    (SplCalcAccs, SplSvcAccParams),
    WsolCalcAccs,
>;

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
        spl_accs(
            keys,
            StatePacked::of_acc_data(&gpc_state.data)
                .unwrap()
                .into_state()
                .last_upgrade_slot,
            SplAccs {
                gpc_state,
                stake_pool,
            },
        ),
    )
}

pub fn svc_accs(params: SvcAccParamsAg) -> (SvcCalcAccsAg, AccountMap) {
    match &params {
        SvcAg::Lido(_) | SvcAg::Marinade(_) => todo!(),
        SvcAg::Inf(_) => (SvcCalcAccsAg::Inf(InfDummyCalcAccs), Default::default()),
        SvcAg::SanctumSpl((_, p)) | SvcAg::SanctumSplMulti((_, p)) | SvcAg::Spl((_, p)) => {
            let (passthrough, keys) = match params {
                SvcAg::SanctumSpl((a, _)) => (SvcAg::SanctumSpl(a), a.suf_keys_owned()),
                SvcAg::SanctumSplMulti((a, _)) => (SvcAg::SanctumSplMulti(a), a.suf_keys_owned()),
                SvcAg::Spl((a, _)) => (SvcAg::Spl(a), a.suf_keys_owned()),
                _ => unreachable!(),
            };
            let keys = IxSufAccs(keys.0.map(Pubkey::from));

            let SplSvcAccParams {
                pool,
                gpc_state,
                last_prog_upg_slot,
            } = p;
            (
                passthrough,
                spl_accs(
                    keys,
                    *last_prog_upg_slot,
                    SplAccs {
                        gpc_state: mock_gpc_state(
                            gpc_state,
                            Pubkey::from(*params.svc_program_id()),
                        ),
                        stake_pool: mock_spl_stake_pool(pool, *keys.pool_prog()),
                    },
                ),
            )
        }
        SvcAg::Wsol(a) => (SvcAg::Wsol(*a), Default::default()),
    }
}

struct SplAccs {
    gpc_state: Account,
    stake_pool: Account,
}

fn spl_accs(
    keys: IxSufAccs<Pubkey>,
    last_prog_upg_slot: u64,
    SplAccs {
        gpc_state,
        stake_pool,
    }: SplAccs,
) -> AccountMap {
    [
        (
            *keys.pool_prog(),
            mock_prog_acc(ProgramDataAddr::Raw(*keys.pool_progdata())),
        ),
        (*keys.pool_progdata(), mock_progdata_acc(last_prog_upg_slot)),
        (*keys.state(), gpc_state),
        (*keys.pool_state(), stake_pool),
    ]
    .into_iter()
    .collect()
}

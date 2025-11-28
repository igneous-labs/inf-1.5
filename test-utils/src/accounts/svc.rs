use inf1_svc_ag_core::{
    inf1_svc_core::traits::SolValCalcAccs, inf1_svc_generic::instructions::IxSufAccs,
    instructions::SvcCalcAccsAg, SvcAg,
};
use inf1_svc_inf_core::InfDummyCalcAccs;
use inf1_svc_spl_core::{
    inf1_svc_generic::accounts::state::State,
    instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
};
use inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs;
use sanctum_spl_stake_pool_core::StakePool;
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::{mock_prog_acc, mock_progdata_acc, AccountMap, ProgramDataAddr};

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

pub fn svc_accs(params: SvcAccParamsAg) -> (SvcCalcAccsAg, AccountMap) {
    match &params {
        SvcAg::Inf(_) => (SvcCalcAccsAg::Inf(InfDummyCalcAccs), Default::default()),
        SvcAg::Lido(_) | SvcAg::Marinade(_) => todo!(),
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
                [
                    (
                        *keys.pool_prog(),
                        mock_prog_acc(ProgramDataAddr::Raw(*keys.pool_progdata())),
                    ),
                    (
                        *keys.pool_progdata(),
                        mock_progdata_acc(*last_prog_upg_slot),
                    ),
                    (
                        *keys.state(),
                        mock_gpc_state(gpc_state, Pubkey::from(*params.svc_program_id())),
                    ),
                    (
                        *keys.pool_state(),
                        mock_spl_stake_pool(pool, *keys.pool_prog()),
                    ),
                ]
                .into_iter()
                .collect(),
            )
        }
        SvcAg::Wsol(a) => (SvcAg::Wsol(*a), Default::default()),
    }
}

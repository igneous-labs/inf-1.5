//! Derive data from accounts

use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_lido_core::{calc::LidoCalc, solido_legacy_core},
    inf1_svc_marinade_core::{calc::MarinadeCalc, sanctum_marinade_liquid_staking_core},
    inf1_svc_spl_core::{
        calc::SplCalc,
        instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
        sanctum_spl_stake_pool_core::StakePool,
    },
    inf1_svc_wsol_core::calc::WsolCalc,
    instructions::SvcCalcAccsAg,
};
use inf1_test_utils::AccountMap;

pub fn derive_svc_no_inf(am: &AccountMap, accs: &SvcCalcAccsAg, curr_epoch: u64) -> SvcCalcAg {
    match accs {
        SvcCalcAccsAg::Wsol(_) => SvcCalcAg::Wsol(WsolCalc),
        SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr })
        | SvcCalcAccsAg::SanctumSpl(SanctumSplCalcAccs { stake_pool_addr })
        | SvcCalcAccsAg::Spl(SplCalcAccs { stake_pool_addr }) => {
            let calc = SplCalc::new(
                &StakePool::borsh_de(am[&(*stake_pool_addr).into()].data.as_slice()).unwrap(),
                curr_epoch,
            );
            match accs {
                SvcCalcAccsAg::SanctumSplMulti(_) => SvcCalcAg::SanctumSplMulti(calc),
                SvcCalcAccsAg::SanctumSpl(_) => SvcCalcAg::SanctumSpl(calc),
                SvcCalcAccsAg::Spl(_) => SvcCalcAg::Spl(calc),
                _ => unreachable!(),
            }
        }
        SvcCalcAccsAg::Marinade(_) => SvcCalcAg::Marinade(MarinadeCalc::new(
            &sanctum_marinade_liquid_staking_core::State::borsh_de(
                am[&sanctum_marinade_liquid_staking_core::STATE_PUBKEY.into()]
                    .data
                    .as_slice(),
            )
            .unwrap(),
        )),
        SvcCalcAccsAg::Lido(_) => SvcCalcAg::Lido(LidoCalc::new(
            &solido_legacy_core::Lido::borsh_de(
                am[&solido_legacy_core::LIDO_STATE_ADDR.into()]
                    .data
                    .as_slice(),
            )
            .unwrap(),
            curr_epoch,
        )),
        SvcCalcAccsAg::Inf(_) => panic!("INF unsupported"),
    }
}

use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::typedefs::lst_state::LstState;
use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_lido_core::{
        calc::LidoCalc,
        instructions::sol_val_calc::LidoCalcAccs,
        solido_legacy_core::{Lido, LIDO_STATE_ADDR},
    },
    inf1_svc_marinade_core::{
        calc::MarinadeCalc, instructions::sol_val_calc::MarinadeCalcAccs,
        sanctum_marinade_liquid_staking_core,
    },
    inf1_svc_spl_core::{
        calc::SplCalc,
        instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
        sanctum_spl_stake_pool_core::{StakePool, SYSVAR_CLOCK},
    },
    inf1_svc_wsol_core::{calc::WsolCalc, instructions::sol_val_calc::WsolCalcAccs},
    instructions::SvcCalcAccsAg,
    SvcAgTy,
};

use crate::{
    acc_deser_err,
    err::{missing_acc_err, missing_spl_data_err, unknown_svc_err, InfError},
    interface::{Account, B58PK},
    utils::epoch_from_clock_data,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calc {
    pub calc: Option<SvcCalcAg>,
    pub accs: SvcCalcAccsAg,
}

impl Calc {
    pub fn new(
        LstState {
            mint,
            sol_value_calculator,
            ..
        }: &LstState,
        spl: &HashMap<[u8; 32], [u8; 32]>,
    ) -> Result<Self, InfError> {
        let ty = SvcAgTy::try_from_program_id(sol_value_calculator)
            .ok_or_else(|| unknown_svc_err(sol_value_calculator))?;
        let stake_pool_addr_res = spl.get(mint).ok_or_else(|| missing_spl_data_err(mint));
        let accs = match ty {
            SvcAgTy::Lido => SvcCalcAccsAg::Lido(LidoCalcAccs),
            SvcAgTy::Marinade => SvcCalcAccsAg::Marinade(MarinadeCalcAccs),
            SvcAgTy::SanctumSpl => SvcCalcAccsAg::SanctumSpl(SanctumSplCalcAccs {
                stake_pool_addr: *stake_pool_addr_res?,
            }),
            SvcAgTy::SanctumSplMulti => SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs {
                stake_pool_addr: *stake_pool_addr_res?,
            }),
            SvcAgTy::Spl => SvcCalcAccsAg::Spl(SplCalcAccs {
                stake_pool_addr: *stake_pool_addr_res?,
            }),
            SvcAgTy::Wsol => SvcCalcAccsAg::Wsol(WsolCalcAccs),
        };
        // special-case:
        // wsol calc doesnt need any additional data, so it can be initialized immediately
        let calc = match ty {
            SvcAgTy::Wsol => Some(SvcCalcAg::Wsol(WsolCalc)),
            _ => None,
        };
        Ok(Self { calc, accs })
    }
}

// TODO: deleteme and move this to inf1-svc-ag-std instead as part of update traits impl
/// Update
impl Calc {
    #[inline]
    pub fn accounts_to_update(&self) -> impl Iterator<Item = [u8; 32]> {
        self.accs.calc_keys()
    }

    #[inline]
    pub fn update(&mut self, fetched: &HashMap<B58PK, Account>) -> Result<(), InfError> {
        let calc = match self.accs {
            SvcCalcAccsAg::Lido(_) => {
                let [lido_acc, clock_acc] = [LIDO_STATE_ADDR, SYSVAR_CLOCK].map(|pk| {
                    fetched
                        .get(&Bs58Array(pk))
                        .ok_or_else(|| missing_acc_err(&pk))
                });
                let lido_acc = lido_acc?;
                let clock_acc = clock_acc?;

                let lido = Lido::borsh_de(lido_acc.data.as_ref())
                    .map_err(|_e| acc_deser_err(&LIDO_STATE_ADDR))?;
                let current_epoch = epoch_from_clock_data(&clock_acc.data)
                    .ok_or_else(|| acc_deser_err(&SYSVAR_CLOCK))?;
                SvcCalcAg::Lido(LidoCalc::new(&lido, current_epoch))
            }
            SvcCalcAccsAg::Marinade(_) => {
                let marinade_acc = fetched
                    .get(&Bs58Array(
                        sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
                    ))
                    .ok_or_else(|| {
                        missing_acc_err(&sanctum_marinade_liquid_staking_core::STATE_PUBKEY)
                    })?;
                let marinade = sanctum_marinade_liquid_staking_core::State::borsh_de(
                    marinade_acc.data.as_ref(),
                )
                .map_err(|_e| acc_deser_err(&sanctum_marinade_liquid_staking_core::STATE_PUBKEY))?;
                SvcCalcAg::Marinade(MarinadeCalc::new(&marinade))
            }
            SvcCalcAccsAg::SanctumSpl(SanctumSplCalcAccs { stake_pool_addr }) => {
                let [pool_acc, clock_acc] = [stake_pool_addr, SYSVAR_CLOCK].map(|pk| {
                    fetched
                        .get(&Bs58Array(pk))
                        .ok_or_else(|| missing_acc_err(&pk))
                });
                let pool_acc = pool_acc?;
                let clock_acc = clock_acc?;

                let pool = StakePool::borsh_de(pool_acc.data.as_ref())
                    .map_err(|_e| acc_deser_err(&stake_pool_addr))?;
                let current_epoch = epoch_from_clock_data(&clock_acc.data)
                    .ok_or_else(|| acc_deser_err(&SYSVAR_CLOCK))?;
                SvcCalcAg::Spl(SplCalc::new(&pool, current_epoch))
            }
            SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }) => {
                let [pool_acc, clock_acc] = [stake_pool_addr, SYSVAR_CLOCK].map(|pk| {
                    fetched
                        .get(&Bs58Array(pk))
                        .ok_or_else(|| missing_acc_err(&pk))
                });
                let pool_acc = pool_acc?;
                let clock_acc = clock_acc?;

                let pool = StakePool::borsh_de(pool_acc.data.as_ref())
                    .map_err(|_e| acc_deser_err(&stake_pool_addr))?;
                let current_epoch = epoch_from_clock_data(&clock_acc.data)
                    .ok_or_else(|| acc_deser_err(&SYSVAR_CLOCK))?;
                SvcCalcAg::SanctumSplMulti(SplCalc::new(&pool, current_epoch))
            }
            SvcCalcAccsAg::Spl(SplCalcAccs { stake_pool_addr }) => {
                let [pool_acc, clock_acc] = [stake_pool_addr, SYSVAR_CLOCK].map(|pk| {
                    fetched
                        .get(&Bs58Array(pk))
                        .ok_or_else(|| missing_acc_err(&pk))
                });
                let pool_acc = pool_acc?;
                let clock_acc = clock_acc?;

                let pool = StakePool::borsh_de(pool_acc.data.as_ref())
                    .map_err(|_e| acc_deser_err(&stake_pool_addr))?;
                let current_epoch = epoch_from_clock_data(&clock_acc.data)
                    .ok_or_else(|| acc_deser_err(&SYSVAR_CLOCK))?;
                SvcCalcAg::Spl(SplCalc::new(&pool, current_epoch))
            }
            SvcCalcAccsAg::Wsol(_) => SvcCalcAg::Wsol(WsolCalc),
        };

        self.calc = Some(calc);

        Ok(())
    }
}

/// SolValCalc traits
impl Calc {
    pub(crate) const fn as_sol_val_calc(&self) -> Option<&SvcCalcAg> {
        self.calc.as_ref()
    }

    pub(crate) const fn as_sol_val_calc_accs(&self) -> &SvcCalcAccsAg {
        &self.accs
    }
}

use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::typedefs::lst_state::LstState;
use inf1_svc_ag::{
    calc::CalcAg,
    inf1_svc_lido_core::{
        calc::LidoCalc,
        solido_legacy_core::{Lido, LIDO_STATE_ADDR},
    },
    inf1_svc_marinade_core::{calc::MarinadeCalc, sanctum_marinade_liquid_staking_core},
    inf1_svc_spl_core::{
        calc::SplCalc,
        instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
        sanctum_spl_stake_pool_core::{StakePool, SYSVAR_CLOCK},
    },
    instructions::{CalcAccsAg, CalcAccsAgTy},
};
use wasm_bindgen::JsError;

use crate::{
    acc_deser_err,
    err::{generic_err, missing_acc_err, missing_spl_data, unknown_svc_err},
    interface::{Account, B58PK},
    utils::epoch_from_clock_data,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calc {
    pub calc: Option<CalcAg>,
    pub accs: CalcAccsAg,
}

impl Calc {
    pub fn new(
        LstState {
            mint,
            sol_value_calculator,
            ..
        }: &LstState,
        spl: &HashMap<B58PK, B58PK>,
    ) -> Result<Self, JsError> {
        let ty = CalcAccsAgTy::try_from_program_id(sol_value_calculator)
            .ok_or_else(|| unknown_svc_err(sol_value_calculator))?;
        let stake_pool_addr_res = spl
            .get(&Bs58Array(*mint))
            .ok_or_else(|| missing_spl_data(mint));
        let accs = match ty {
            CalcAccsAgTy::Lido => CalcAccsAg::Lido,
            CalcAccsAgTy::Marinade => CalcAccsAg::Marinade,
            CalcAccsAgTy::SanctumSpl => CalcAccsAg::SanctumSpl(SanctumSplCalcAccs {
                stake_pool_addr: stake_pool_addr_res?.0,
            }),
            CalcAccsAgTy::SanctumSplMulti => CalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs {
                stake_pool_addr: stake_pool_addr_res?.0,
            }),
            CalcAccsAgTy::Spl => CalcAccsAg::Spl(SplCalcAccs {
                stake_pool_addr: stake_pool_addr_res?.0,
            }),
            CalcAccsAgTy::Wsol => CalcAccsAg::Wsol,
        };
        Ok(Self { calc: None, accs })
    }
}

/// Update
impl Calc {
    #[inline]
    pub fn accounts_to_update(&self) -> impl Iterator<Item = &[u8; 32]> {
        self.accs.calc_keys()
    }

    #[inline]
    pub fn update(&mut self, fetched: &HashMap<B58PK, Account>) -> Result<(), JsError> {
        let calc = match self.accs {
            CalcAccsAg::Lido => {
                let [lido_acc, clock_acc] = [LIDO_STATE_ADDR, SYSVAR_CLOCK].map(|pk| {
                    fetched
                        .get(&Bs58Array(pk))
                        .ok_or_else(|| missing_acc_err(&pk))
                });
                let lido_acc = lido_acc?;
                let clock_acc = clock_acc?;

                let lido = Lido::borsh_de(lido_acc.data.as_ref()).map_err(generic_err)?;
                let current_epoch = epoch_from_clock_data(&clock_acc.data)
                    .ok_or_else(|| acc_deser_err(&SYSVAR_CLOCK))?;
                Some(CalcAg::Lido(LidoCalc::new(&lido, current_epoch)))
            }
            CalcAccsAg::Marinade => {
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
                .map_err(generic_err)?;
                Some(CalcAg::Marinade(MarinadeCalc::new(&marinade)))
            }
            CalcAccsAg::SanctumSpl(SanctumSplCalcAccs { stake_pool_addr })
            | CalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr })
            | CalcAccsAg::Spl(SplCalcAccs { stake_pool_addr }) => {
                let [pool_acc, clock_acc] = [stake_pool_addr, SYSVAR_CLOCK].map(|pk| {
                    fetched
                        .get(&Bs58Array(pk))
                        .ok_or_else(|| missing_acc_err(&pk))
                });
                let pool_acc = pool_acc?;
                let clock_acc = clock_acc?;

                let pool = StakePool::borsh_de(pool_acc.data.as_ref()).map_err(generic_err)?;
                let current_epoch = epoch_from_clock_data(&clock_acc.data)
                    .ok_or_else(|| acc_deser_err(&SYSVAR_CLOCK))?;
                Some(CalcAg::Spl(SplCalc::new(&pool, current_epoch)))
            }
            CalcAccsAg::Wsol => None,
        };

        self.calc = calc;

        Ok(())
    }
}

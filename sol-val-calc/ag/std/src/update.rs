use std::{
    error::Error,
    fmt::{Display, Formatter},
    iter::{empty, once},
};

use inf1_svc_ag_core::{
    inf1_svc_lido_core::{
        calc::LidoCalc,
        solido_legacy_core::{Lido, LIDO_STATE_ADDR, SYSVAR_CLOCK},
    },
    inf1_svc_marinade_core::{self, calc::MarinadeCalc, sanctum_marinade_liquid_staking_core},
    inf1_svc_spl_core::{
        calc::SplCalc,
        instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
        sanctum_spl_stake_pool_core::StakePool,
    },
    inf1_svc_wsol_core::calc::WsolCalc,
    SvcAg,
};

use crate::{SvcAgStd, SvcCalcAccsPair};

// Re-exports
pub use inf1_svc_std::update::{Account, AccountsToUpdateSvc, UpdateErr, UpdateMap, UpdateSvc};

type LidoPkIter = core::array::IntoIter<[u8; 32], 2>;
type MarinadePkIter = core::iter::Once<[u8; 32]>;
type SanctumSplPkIter = core::array::IntoIter<[u8; 32], 2>;
type SanctumSplMultiPkIter = core::array::IntoIter<[u8; 32], 2>;
type SplPkIter = core::array::IntoIter<[u8; 32], 2>;
type WsolPkIter = core::iter::Empty<[u8; 32]>;

pub type SvcPkIterAg = SvcAg<
    LidoPkIter,
    MarinadePkIter,
    SanctumSplPkIter,
    SanctumSplMultiPkIter,
    SplPkIter,
    WsolPkIter,
>;

impl AccountsToUpdateSvc for SvcAgStd {
    type PkIter = SvcPkIterAg;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        match self.as_sol_val_calc_accs() {
            SvcAg::Lido(_) => SvcAg::Lido([LIDO_STATE_ADDR, SYSVAR_CLOCK].into_iter()),
            SvcAg::Marinade(_) => SvcAg::Marinade(once(
                inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
            )),
            SvcAg::SanctumSpl(SanctumSplCalcAccs { stake_pool_addr }) => {
                SvcAg::SanctumSpl([*stake_pool_addr, SYSVAR_CLOCK].into_iter())
            }
            SvcAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }) => {
                SvcAg::SanctumSplMulti([*stake_pool_addr, SYSVAR_CLOCK].into_iter())
            }
            SvcAg::Spl(SplCalcAccs { stake_pool_addr }) => {
                SvcAg::Spl([*stake_pool_addr, SYSVAR_CLOCK].into_iter())
            }
            SvcAg::Wsol(_) => SvcAg::Wsol(empty()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvcCommonUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for SvcCommonUpdateErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for SvcCommonUpdateErr {}

pub type UpdateSvcErr = SvcAg<
    SvcCommonUpdateErr,
    SvcCommonUpdateErr,
    SvcCommonUpdateErr,
    SvcCommonUpdateErr,
    SvcCommonUpdateErr,
    SvcCommonUpdateErr,
>;

impl UpdateSvc for SvcAgStd {
    type InnerErr = UpdateSvcErr;

    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            SvcAg::Lido(SvcCalcAccsPair { calc, .. }) => {
                let [lido_acc, clock_acc] =
                    [LIDO_STATE_ADDR, SYSVAR_CLOCK].map(|pk| update_map.get_account_checked(&pk));
                let lido_acc = lido_acc?;
                let clock_acc = clock_acc?;

                let lido = Lido::borsh_de(lido_acc.data()).map_err(|_e| {
                    UpdateErr::Inner(SvcAg::Lido(SvcCommonUpdateErr::AccDeser {
                        pk: LIDO_STATE_ADDR,
                    }))
                })?;
                let current_epoch = epoch_from_clock_data(clock_acc.data()).ok_or({
                    UpdateErr::Inner(SvcAg::Lido(SvcCommonUpdateErr::AccDeser {
                        pk: SYSVAR_CLOCK,
                    }))
                })?;

                *calc = Some(LidoCalc::new(&lido, current_epoch));
            }
            SvcAg::Marinade(SvcCalcAccsPair { calc, .. }) => {
                let marinade_acc = update_map
                    .get_account_checked(&sanctum_marinade_liquid_staking_core::STATE_PUBKEY)?;
                let marinade =
                    sanctum_marinade_liquid_staking_core::State::borsh_de(marinade_acc.data())
                        .map_err(|_e| {
                            UpdateErr::Inner(SvcAg::Marinade(SvcCommonUpdateErr::AccDeser {
                                pk: sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
                            }))
                        })?;
                *calc = Some(MarinadeCalc::new(&marinade));
            }
            SvcAg::SanctumSpl(SvcCalcAccsPair {
                calc,
                accs: SanctumSplCalcAccs { stake_pool_addr },
            }) => {
                let spl_calc = updated_spl_calc(*stake_pool_addr, update_map, SvcAg::SanctumSpl)?;
                *calc = Some(spl_calc);
            }
            SvcAg::SanctumSplMulti(SvcCalcAccsPair {
                calc,
                accs: SanctumSplMultiCalcAccs { stake_pool_addr },
            }) => {
                let spl_calc =
                    updated_spl_calc(*stake_pool_addr, update_map, SvcAg::SanctumSplMulti)?;
                *calc = Some(spl_calc);
            }
            SvcAg::Spl(SvcCalcAccsPair {
                calc,
                accs: SplCalcAccs { stake_pool_addr },
            }) => {
                let spl_calc = updated_spl_calc(*stake_pool_addr, update_map, SvcAg::Spl)?;
                *calc = Some(spl_calc);
            }
            SvcAg::Wsol(SvcCalcAccsPair { calc, .. }) => *calc = Some(WsolCalc),
        }

        Ok(())
    }
}

fn updated_spl_calc(
    stake_pool_addr: [u8; 32],
    update_map: impl UpdateMap,
    variant: impl Fn(SvcCommonUpdateErr) -> UpdateSvcErr,
) -> Result<SplCalc, UpdateErr<UpdateSvcErr>> {
    let [pool_acc, clock_acc] =
        [stake_pool_addr, SYSVAR_CLOCK].map(|pk| update_map.get_account_checked(&pk));
    let pool_acc = pool_acc?;
    let clock_acc = clock_acc?;

    let pool = StakePool::borsh_de(pool_acc.data()).map_err(|_e| {
        UpdateErr::Inner(variant(SvcCommonUpdateErr::AccDeser {
            pk: stake_pool_addr,
        }))
    })?;
    let current_epoch = epoch_from_clock_data(clock_acc.data()).ok_or_else(|| {
        UpdateErr::Inner(variant(SvcCommonUpdateErr::AccDeser { pk: SYSVAR_CLOCK }))
    })?;
    Ok(SplCalc::new(&pool, current_epoch))
}

fn epoch_from_clock_data(clock_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(clock_acc_data, 16)
}

fn u64_le_at(data: &[u8], at: usize) -> Option<u64> {
    chunk_at(data, at).map(|c| u64::from_le_bytes(*c))
}

fn chunk_at<const N: usize>(data: &[u8], at: usize) -> Option<&[u8; N]> {
    data.get(at..).and_then(|s| s.first_chunk())
}

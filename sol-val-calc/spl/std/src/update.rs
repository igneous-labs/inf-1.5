use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use inf1_svc_spl_core::{
    calc::SplCalc,
    sanctum_spl_stake_pool_core::{StakePool, SYSVAR_CLOCK},
};

use crate::{SanctumSplMultiSvcStd, SanctumSplSvcStd, SplSvcStd};

// Re-exports
pub use inf1_svc_std::update::*;

pub type PkIter = core::array::IntoIter<[u8; 32], 2>;

impl AccountsToUpdateSvc for SanctumSplSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        [self.accs.stake_pool_addr, SYSVAR_CLOCK].into_iter()
    }
}

impl AccountsToUpdateSvc for SanctumSplMultiSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        [self.accs.stake_pool_addr, SYSVAR_CLOCK].into_iter()
    }
}

impl AccountsToUpdateSvc for SplSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        [self.accs.stake_pool_addr, SYSVAR_CLOCK].into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SplUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for SplUpdateErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for SplUpdateErr {}

fn updated_spl_calc(
    stake_pool_addr: [u8; 32],
    update_map: impl UpdateMap,
) -> Result<SplCalc, UpdateErr<SplUpdateErr>> {
    let pool = fetched_stake_pool(&stake_pool_addr, &update_map)?;
    let clock_acc = update_map.get_account_checked(&SYSVAR_CLOCK)?;
    let current_epoch = epoch_from_clock_data(clock_acc.data()).ok_or(UpdateErr::Inner(
        SplUpdateErr::AccDeser { pk: SYSVAR_CLOCK },
    ))?;
    Ok(SplCalc::new(&pool, current_epoch))
}

fn fetched_stake_pool(
    stake_pool_addr: &[u8; 32],
    update_map: impl UpdateMap,
) -> Result<StakePool, UpdateErr<SplUpdateErr>> {
    let pool_acc = update_map.get_account_checked(stake_pool_addr)?;
    StakePool::borsh_de(pool_acc.data()).map_err(|_e| {
        UpdateErr::Inner(SplUpdateErr::AccDeser {
            pk: *stake_pool_addr,
        })
    })
}

impl UpdateSvc for SanctumSplSvcStd {
    type InnerErr = SplUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.calc = Some(updated_spl_calc(self.accs.stake_pool_addr, update_map)?);
        Ok(())
    }
}

impl UpdateSvc for SanctumSplMultiSvcStd {
    type InnerErr = SplUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.calc = Some(updated_spl_calc(self.accs.stake_pool_addr, update_map)?);
        Ok(())
    }
}

impl UpdateSvc for SplSvcStd {
    type InnerErr = SplUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.calc = Some(updated_spl_calc(self.accs.stake_pool_addr, update_map)?);
        Ok(())
    }
}

impl SanctumSplSvcStd {
    /// Update, but exclude data derived from clock
    /// (currently just `current_epoch`).
    ///
    /// Such data is retained unchanged if existing data exists,
    /// otherwise set to default.
    ///
    /// Required to workaround jup special-casing clock.
    #[inline]
    pub fn update_svc_no_clock(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<SplUpdateErr>> {
        let current_epoch = self.calc.map(|c| c.current_epoch).unwrap_or_default();
        self.calc = Some(SplCalc::new(
            &fetched_stake_pool(&self.accs.stake_pool_addr, update_map)?,
            current_epoch,
        ));
        Ok(())
    }
}

impl SanctumSplMultiSvcStd {
    /// Update, but exclude data derived from clock
    /// (currently just `current_epoch`).
    ///
    /// Such data is retained unchanged if existing data exists,
    /// otherwise set to default.
    ///
    /// Required to workaround jup special-casing clock.
    #[inline]
    pub fn update_svc_no_clock(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<SplUpdateErr>> {
        let current_epoch = self.calc.map(|c| c.current_epoch).unwrap_or_default();
        self.calc = Some(SplCalc::new(
            &fetched_stake_pool(&self.accs.stake_pool_addr, update_map)?,
            current_epoch,
        ));
        Ok(())
    }
}

impl SplSvcStd {
    /// Update, but exclude data derived from clock
    /// (currently just `current_epoch`).
    ///
    /// Such data is retained unchanged if existing data exists,
    /// otherwise set to default.
    ///
    /// Required to workaround jup special-casing clock.
    #[inline]
    pub fn update_svc_no_clock(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<SplUpdateErr>> {
        let current_epoch = self.calc.map(|c| c.current_epoch).unwrap_or_default();
        self.calc = Some(SplCalc::new(
            &fetched_stake_pool(&self.accs.stake_pool_addr, update_map)?,
            current_epoch,
        ));
        Ok(())
    }
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

use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use inf1_svc_lido_core::{
    calc::LidoCalc,
    solido_legacy_core::{Lido, LIDO_STATE_ADDR, SYSVAR_CLOCK},
};

use crate::LidoSvcStd;

// Re-exports
pub use inf1_svc_std::update::*;

pub type PkIter = core::array::IntoIter<[u8; 32], 2>;

impl AccountsToUpdateSvc for LidoSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        [LIDO_STATE_ADDR, SYSVAR_CLOCK].into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LidoUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for LidoUpdateErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for LidoUpdateErr {}

impl UpdateSvc for LidoSvcStd {
    type InnerErr = LidoUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        let lido = fetched_lido(&update_map)?;
        let clock_acc = update_map.get_account_checked(&SYSVAR_CLOCK)?;
        let current_epoch = epoch_from_clock_data(clock_acc.data()).ok_or(UpdateErr::Inner(
            LidoUpdateErr::AccDeser { pk: SYSVAR_CLOCK },
        ))?;

        self.calc = Some(LidoCalc::new(&lido, current_epoch));

        Ok(())
    }
}

impl LidoSvcStd {
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
    ) -> Result<(), UpdateErr<LidoUpdateErr>> {
        let lido = fetched_lido(update_map)?;
        let current_epoch = self.calc.map(|c| c.current_epoch).unwrap_or_default();
        self.calc = Some(LidoCalc::new(&lido, current_epoch));
        Ok(())
    }
}

fn fetched_lido(update_map: impl UpdateMap) -> Result<Lido, UpdateErr<LidoUpdateErr>> {
    let lido_acc = update_map.get_account_checked(&LIDO_STATE_ADDR)?;
    Lido::borsh_de(lido_acc.data()).map_err(|_e| {
        UpdateErr::Inner(LidoUpdateErr::AccDeser {
            pk: LIDO_STATE_ADDR,
        })
    })
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

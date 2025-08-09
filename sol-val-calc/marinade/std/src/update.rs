use std::{
    error::Error,
    fmt::{Display, Formatter},
    iter::once,
};

use inf1_svc_marinade_core::{
    calc::MarinadeCalc,
    sanctum_marinade_liquid_staking_core::{State, STATE_PUBKEY},
};

use crate::MarinadeSvcStd;

// Re-exports
pub use inf1_svc_std::update::*;

pub type PkIter = core::iter::Once<[u8; 32]>;

impl AccountsToUpdateSvc for MarinadeSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        once(STATE_PUBKEY)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarinadeUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for MarinadeUpdateErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for MarinadeUpdateErr {}

impl UpdateSvc for MarinadeSvcStd {
    type InnerErr = MarinadeUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        let marinade_acc = update_map.get_account_checked(&STATE_PUBKEY)?;
        let marinade = State::borsh_de(marinade_acc.data())
            .map_err(|_e| UpdateErr::Inner(MarinadeUpdateErr::AccDeser { pk: STATE_PUBKEY }))?;

        self.calc = Some(MarinadeCalc::new(&marinade));

        Ok(())
    }
}

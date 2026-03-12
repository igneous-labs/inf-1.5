use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use inf1_ctl_core::{
    accounts::pool_state::VerPoolState,
    keys::POOL_STATE_ID,
    svc::{InfCalc, InfDummyCalcAccs},
};
use inf1_svc_std::update::{Account, AccountsToUpdateSvc, UpdateErr, UpdateMap, UpdateSvc};

// Re-exports
pub use inf1_ctl_core::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfSvcStd {
    pub calc: InfCalc,

    // FIXME? this mint addr will probably be duplicated in
    // most contexts with the one stored in an accompanying PoolState
    pub mint_addr: [u8; 32],
}

pub type PkIter = core::array::IntoIter<[u8; 32], 2>;

impl AccountsToUpdateSvc for InfSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        self.atus_accs_to_update_svc().into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for InfUpdateErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for InfUpdateErr {}

impl UpdateSvc for InfSvcStd {
    type InnerErr = InfUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.us_update_svc(update_map)
    }
}

impl InfSvcStd {
    pub const DEFAULT: Self = Self {
        calc: InfCalc::DEFAULT,
        mint_addr: [0u8; 32],
    };

    #[inline]
    pub const fn atus_accs_to_update_svc(&self) -> [[u8; 32]; 2] {
        [POOL_STATE_ID, self.mint_addr]
    }

    #[inline]
    pub fn us_update_svc(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfUpdateErr>> {
        let [pool_addr, mint_addr] = self.atus_accs_to_update_svc();
        let [p, m] = [pool_addr, mint_addr].map(|a| update_map.get_account_checked(&a));
        let pool_state_acc = p?;
        let lp_mint_acc = m?;

        let pool_state_v2 = VerPoolState::try_from_acc_data(pool_state_acc.data())
            .ok_or(UpdateErr::Inner(InfUpdateErr::AccDeser { pk: pool_addr }))?
            .migrated(0 /* migration_slot has no effect here */);

        let inf_mint_supply = token_supply_from_mint_data(lp_mint_acc.data())
            .ok_or(UpdateErr::Inner(InfUpdateErr::AccDeser { pk: mint_addr }))?;

        self.calc = InfCalc::new(&pool_state_v2, inf_mint_supply);

        Ok(())
    }
}

/// Accessors
impl InfSvcStd {
    #[inline]
    pub const fn as_calc(&self) -> &InfCalc {
        &self.calc
    }

    #[inline]
    pub const fn as_accs(&self) -> &InfDummyCalcAccs {
        &InfDummyCalcAccs
    }
}

// TODO below util fns are duplicated in inf1-std

fn token_supply_from_mint_data(mint_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(mint_acc_data, 36)
}

fn u64_le_at(data: &[u8], at: usize) -> Option<u64> {
    chunk_at(data, at).map(|c| u64::from_le_bytes(*c))
}

fn chunk_at<const N: usize>(data: &[u8], at: usize) -> Option<&[u8; N]> {
    data.get(at..).and_then(|s| s.first_chunk())
}

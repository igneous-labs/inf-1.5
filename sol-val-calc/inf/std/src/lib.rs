use std::convert::Infallible;

use inf1_svc_std::update::{AccountsToUpdateSvc, UpdateSvc};

// Re-exports
pub use inf1_svc_inf_core::*;

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
        [POOL_STATE_ID, self.mint_addr].into_iter()
    }
}

impl UpdateSvc for InfSvcStd {
    type InnerErr = Infallible;

    fn update_svc(
        &mut self,
        _update_map: impl inf1_svc_std::update::UpdateMap,
    ) -> Result<(), inf1_svc_std::update::UpdateErr<Self::InnerErr>> {
        todo!()
    }
}

impl InfSvcStd {
    pub const DEFAULT: Self = Self {
        calc: InfCalc::DEFAULT,
        mint_addr: [0u8; 32],
    };
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

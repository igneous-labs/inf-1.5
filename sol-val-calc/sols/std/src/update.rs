use std::{convert::Infallible, iter::empty};

use crate::SolsSvcStd;

// Re-exports
pub use inf1_svc_std::update::*;

pub type PkIter = core::iter::Empty<[u8; 32]>;

impl AccountsToUpdateSvc for SolsSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        empty()
    }
}

pub type SolsUpdateErr = Infallible;

impl UpdateSvc for SolsSvcStd {
    type InnerErr = SolsUpdateErr;

    #[inline]
    fn update_svc(&mut self, _update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        Ok(())
    }
}

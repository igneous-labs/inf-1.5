use std::{convert::Infallible, iter::empty};

use inf1_svc_std::update::{AccountsToUpdateSvc, UpdateErr, UpdateMap, UpdateSvc};

use crate::WsolSvcStd;

pub type PkIter = core::iter::Empty<[u8; 32]>;

impl AccountsToUpdateSvc for WsolSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        empty()
    }
}

impl UpdateSvc for WsolSvcStd {
    type InnerErr = Infallible;

    #[inline]
    fn update_svc(&mut self, _update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        Ok(())
    }
}

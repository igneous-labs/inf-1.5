use std::error::Error;

// Re-exports
pub use inf1_update_traits::{Account, UpdateErr, UpdateMap};

pub trait AccountsToUpdateSvc {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_svc(&self) -> Self::PkIter;
}

pub trait UpdateSvc {
    type InnerErr: Error;

    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>>;
}

// TODO: might need a new trait if a different set of accounts
// compared to quoting/calc-ing is required for instruction formation
// for some svc programs

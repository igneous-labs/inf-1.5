use inf1_svc_spl_core::{
    calc::SplCalc,
    instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
};

// Re-exports
pub use inf1_svc_spl_core::*;

pub mod update;

pub type SanctumSplSvcStd = GenSplSvcStd<SanctumSplCalcAccs>;
pub type SanctumSplMultiSvcStd = GenSplSvcStd<SanctumSplMultiCalcAccs>;
pub type SplSvcStd = GenSplSvcStd<SplCalcAccs>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenSplSvcStd<A> {
    /// Might be `None` at initialization before accounts required
    /// to create the calc have been fetched
    pub calc: Option<SplCalc>,
    pub accs: A,
}

/// Constructors
impl SanctumSplSvcStd {
    pub const fn new(stake_pool_addr: [u8; 32]) -> Self {
        Self {
            calc: None,
            accs: SanctumSplCalcAccs { stake_pool_addr },
        }
    }
}

/// Constructors
impl SanctumSplMultiSvcStd {
    pub const fn new(stake_pool_addr: [u8; 32]) -> Self {
        Self {
            calc: None,
            accs: SanctumSplMultiCalcAccs { stake_pool_addr },
        }
    }
}

/// Constructors
impl SplSvcStd {
    pub const fn new(stake_pool_addr: [u8; 32]) -> Self {
        Self {
            calc: None,
            accs: SplCalcAccs { stake_pool_addr },
        }
    }
}

/// Accessors
impl<A> GenSplSvcStd<A> {
    #[inline]
    pub const fn as_calc(&self) -> Option<&SplCalc> {
        self.calc.as_ref()
    }

    #[inline]
    pub const fn as_accs(&self) -> &A {
        &self.accs
    }
}

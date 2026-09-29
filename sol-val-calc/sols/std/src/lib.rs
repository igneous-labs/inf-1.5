use inf1_svc_sols_core::{calc::SolsCalc, instructions::sol_val_calc::SolsCalcAccs};

// Re-exports
pub use inf1_svc_sols_core::*;

pub mod update;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SolsSvcStd {
    pub sols_pool_addr: [u8; 32],
}

/// Accessors
impl SolsSvcStd {
    #[inline]
    pub const fn as_calc(&self) -> &SolsCalc {
        &SolsCalc
    }

    #[inline]
    pub const fn as_accs(&self) -> &SolsCalcAccs {
        SolsCalcAccs::of_sols_pool_addr(&self.sols_pool_addr)
    }
}

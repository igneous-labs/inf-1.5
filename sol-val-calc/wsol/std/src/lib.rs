use inf1_svc_wsol_core::{calc::WsolCalc, instructions::sol_val_calc::WsolCalcAccs};

// Re-exports
pub use inf1_svc_wsol_core::*;

pub mod update;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WsolSvcStd;

/// Accessors
impl WsolSvcStd {
    #[inline]
    pub const fn as_calc(&self) -> &WsolCalc {
        &WsolCalc
    }

    #[inline]
    pub const fn as_accs(&self) -> &WsolCalcAccs {
        &WsolCalcAccs
    }
}

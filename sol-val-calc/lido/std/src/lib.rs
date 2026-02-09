use inf1_svc_lido_core::{calc::LidoCalc, instructions::sol_val_calc::LidoCalcAccs};

// Re-exports
pub use inf1_svc_lido_core::*;

pub mod update;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LidoSvcStd {
    /// Might be `None` at initialization before accounts required
    /// to create the calc have been fetched
    pub calc: Option<LidoCalc>,
}

impl Default for LidoSvcStd {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Constructors
impl LidoSvcStd {
    pub const DEFAULT: Self = Self { calc: None };
}

/// Accessors
impl LidoSvcStd {
    #[inline]
    pub const fn as_calc(&self) -> Option<&LidoCalc> {
        self.calc.as_ref()
    }

    #[inline]
    pub const fn as_accs(&self) -> &LidoCalcAccs {
        &LidoCalcAccs
    }
}

use inf1_svc_marinade_core::{calc::MarinadeCalc, instructions::sol_val_calc::MarinadeCalcAccs};

// Re-exports
pub use inf1_svc_marinade_core::*;

pub mod update;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarinadeSvcStd {
    /// Might be `None` at initialization before accounts required
    /// to create the calc have been fetched
    pub calc: Option<MarinadeCalc>,
}

impl Default for MarinadeSvcStd {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Constructors
impl MarinadeSvcStd {
    pub const DEFAULT: Self = Self { calc: None };
}

/// Accessors
impl MarinadeSvcStd {
    #[inline]
    pub const fn as_calc(&self) -> Option<&MarinadeCalc> {
        self.calc.as_ref()
    }

    #[inline]
    pub const fn as_accs(&self) -> &MarinadeCalcAccs {
        &MarinadeCalcAccs
    }
}

use inf1_svc_lido_core::{calc::LidoCalc, instructions::sol_val_calc::LidoCalcAccs};

// Re-exports
pub use inf1_svc_lido_core::*;

pub mod update;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LidoCalcAccsPair {
    /// Might be `None` at initialization before accounts required
    /// to create the calc have been fetched
    calc: Option<LidoCalc>,
    accs: LidoCalcAccs,
}

impl Default for LidoCalcAccsPair {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Constructors
impl LidoCalcAccsPair {
    pub const DEFAULT: Self = Self {
        calc: None,
        accs: LidoCalcAccs,
    };
}

/// Accessors
impl LidoCalcAccsPair {
    #[inline]
    pub const fn as_calc(&self) -> Option<&LidoCalc> {
        self.calc.as_ref()
    }

    #[inline]
    pub const fn as_accs(&self) -> &LidoCalcAccs {
        &self.accs
    }
}

use inf1_svc_ag_core::{
    calc::SvcCalcAgRef,
    inf1_svc_lido_core::{calc::LidoCalc, instructions::sol_val_calc::LidoCalcAccs},
    inf1_svc_marinade_core::{calc::MarinadeCalc, instructions::sol_val_calc::MarinadeCalcAccs},
    inf1_svc_spl_core::{
        calc::SplCalc,
        instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
    },
    inf1_svc_wsol_core::{calc::WsolCalc, instructions::sol_val_calc::WsolCalcAccs},
    instructions::SvcCalcAccsAgRef,
};

// Re-exports
pub use inf1_svc_ag_core::*;

pub mod update;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvcCalcAccsPair<C, A> {
    /// Might be `None` at initialization before accounts required
    /// to create the calc have been fetched
    calc: Option<C>,
    accs: A,
}

impl<C, A> SvcCalcAccsPair<C, A> {
    #[inline]
    pub const fn as_calc(&self) -> Option<&C> {
        self.calc.as_ref()
    }

    #[inline]
    pub const fn as_accs(&self) -> &A {
        &self.accs
    }
}

pub type LidoSvcCalcAccsPair = SvcCalcAccsPair<LidoCalc, LidoCalcAccs>;
pub type MarinadeCalcAccsPair = SvcCalcAccsPair<MarinadeCalc, MarinadeCalcAccs>;
pub type SanctumSplSvcCalcAccsPair = SvcCalcAccsPair<SplCalc, SanctumSplCalcAccs>;
pub type SanctumSplMultiSvcCalcAccsPair = SvcCalcAccsPair<SplCalc, SanctumSplMultiCalcAccs>;
pub type SplSvcCalcAccsPair = SvcCalcAccsPair<SplCalc, SplCalcAccs>;
pub type WsolSvcCalcAccsPair = SvcCalcAccsPair<WsolCalc, WsolCalcAccs>;

// simple newtype to workaround orphan rules
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct SvcAgStd(
    pub  SvcAg<
        LidoSvcCalcAccsPair,
        MarinadeCalcAccsPair,
        SanctumSplSvcCalcAccsPair,
        SanctumSplMultiSvcCalcAccsPair,
        SplSvcCalcAccsPair,
        WsolSvcCalcAccsPair,
    >,
);

/// Type alias just to be explicit about what this pubkey is supposed to be
pub type StakePoolAddr = [u8; 32];

pub type SvcCalcStdInitData = SvcAg<(), (), StakePoolAddr, StakePoolAddr, StakePoolAddr, ()>;

/// Constructors
impl SvcAgStd {
    #[inline]
    pub const fn new(init: SvcCalcStdInitData) -> Self {
        Self(match init {
            SvcAg::Lido(_) => SvcAg::Lido(SvcCalcAccsPair {
                calc: None,
                accs: LidoCalcAccs,
            }),
            SvcAg::Marinade(_) => SvcAg::Marinade(SvcCalcAccsPair {
                calc: None,
                accs: MarinadeCalcAccs,
            }),
            SvcAg::SanctumSpl(stake_pool_addr) => SvcAg::SanctumSpl(SvcCalcAccsPair {
                calc: None,
                accs: SanctumSplCalcAccs { stake_pool_addr },
            }),
            SvcAg::SanctumSplMulti(stake_pool_addr) => SvcAg::SanctumSplMulti(SvcCalcAccsPair {
                calc: None,
                accs: SanctumSplMultiCalcAccs { stake_pool_addr },
            }),
            SvcAg::Spl(stake_pool_addr) => SvcAg::Spl(SvcCalcAccsPair {
                calc: None,
                accs: SplCalcAccs { stake_pool_addr },
            }),
            SvcAg::Wsol(_) => SvcAg::Wsol(SvcCalcAccsPair {
                // special-case:
                // wsol calc doesnt need any additional data, so it can be initialized immediately
                calc: Some(WsolCalc),
                accs: WsolCalcAccs,
            }),
        })
    }
}

/// SolValCalc traits
impl SvcAgStd {
    #[inline]
    pub const fn as_sol_val_calc(&self) -> Option<SvcCalcAgRef> {
        match &self.0 {
            SvcAg::Lido(c) => match c.as_calc() {
                Some(r) => Some(SvcAg::Lido(r)),
                None => None,
            },
            SvcAg::Marinade(c) => match c.as_calc() {
                Some(r) => Some(SvcAg::Marinade(r)),
                None => None,
            },
            SvcAg::SanctumSpl(c) => match c.as_calc() {
                Some(r) => Some(SvcAg::SanctumSpl(r)),
                None => None,
            },
            SvcAg::SanctumSplMulti(c) => match c.as_calc() {
                Some(r) => Some(SvcAg::SanctumSplMulti(r)),
                None => None,
            },
            SvcAg::Spl(c) => match c.as_calc() {
                Some(r) => Some(SvcAg::Spl(r)),
                None => None,
            },
            SvcAg::Wsol(c) => match c.as_calc() {
                Some(r) => Some(SvcAg::Wsol(r)),
                None => None,
            },
        }
    }

    #[inline]
    pub const fn as_sol_val_calc_accs(&self) -> SvcCalcAccsAgRef {
        match &self.0 {
            SvcAg::Lido(c) => SvcAg::Lido(c.as_accs()),
            SvcAg::Marinade(c) => SvcAg::Marinade(c.as_accs()),
            SvcAg::SanctumSpl(c) => SvcAg::SanctumSpl(c.as_accs()),
            SvcAg::SanctumSplMulti(c) => SvcAg::SanctumSplMulti(c.as_accs()),
            SvcAg::Spl(c) => SvcAg::Spl(c.as_accs()),
            SvcAg::Wsol(c) => SvcAg::Wsol(c.as_accs()),
        }
    }
}

use inf1_svc_ag_core::{calc::SvcCalcAgRef, instructions::SvcCalcAccsAgRef};

use inf1_svc_lido_std::LidoSvcStd;
use inf1_svc_marinade_std::MarinadeSvcStd;
use inf1_svc_spl_std::{SanctumSplMultiSvcStd, SanctumSplSvcStd, SplSvcStd};
use inf1_svc_wsol_std::WsolSvcStd;

// Re-exports
pub use inf1_svc_ag_core::*;
pub use inf1_svc_lido_std;
pub use inf1_svc_marinade_std;
pub use inf1_svc_spl_std;
pub use inf1_svc_wsol_std;

pub mod update;

// simple newtype to workaround orphan rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SvcAgStd(
    pub  SvcAg<
        LidoSvcStd,
        MarinadeSvcStd,
        SanctumSplSvcStd,
        SanctumSplMultiSvcStd,
        SplSvcStd,
        WsolSvcStd,
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
            SvcAg::Lido(_) => SvcAg::Lido(LidoSvcStd::DEFAULT),
            SvcAg::Marinade(_) => SvcAg::Marinade(MarinadeSvcStd::DEFAULT),
            SvcAg::SanctumSpl(stake_pool_addr) => {
                SvcAg::SanctumSpl(SanctumSplSvcStd::new(stake_pool_addr))
            }
            SvcAg::SanctumSplMulti(stake_pool_addr) => {
                SvcAg::SanctumSplMulti(SanctumSplMultiSvcStd::new(stake_pool_addr))
            }
            SvcAg::Spl(stake_pool_addr) => SvcAg::Spl(SplSvcStd::new(stake_pool_addr)),
            SvcAg::Wsol(_) => SvcAg::Wsol(WsolSvcStd),
        })
    }
}

/// SolValCalc traits
impl SvcAgStd {
    #[inline]
    pub const fn as_sol_val_calc(&self) -> Option<SvcCalcAgRef<'_>> {
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
            SvcAg::Wsol(c) => Some(SvcAg::Wsol(c.as_calc())),
        }
    }

    #[inline]
    pub const fn as_sol_val_calc_accs(&self) -> SvcCalcAccsAgRef<'_> {
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

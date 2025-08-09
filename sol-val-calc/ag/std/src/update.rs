use inf1_svc_ag_core::SvcAg;

use crate::SvcAgStd;

// Re-exports
pub use inf1_svc_lido_std::update::{LidoUpdateErr, PkIter as LidoPkIter};
pub use inf1_svc_marinade_std::update::{MarinadeUpdateErr, PkIter as MarinadePkIter};
pub use inf1_svc_spl_std::update::{PkIter as SplPkIter, SplUpdateErr};
pub use inf1_svc_std::update::*;
pub use inf1_svc_wsol_std::update::{PkIter as WsolPkIter, WsolUpdateErr};

pub type SvcPkIterAg =
    SvcAg<LidoPkIter, MarinadePkIter, SplPkIter, SplPkIter, SplPkIter, WsolPkIter>;

impl AccountsToUpdateSvc for SvcAgStd {
    type PkIter = SvcPkIterAg;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        match self.0 {
            SvcAg::Lido(s) => SvcAg::Lido(s.accounts_to_update_svc()),
            SvcAg::Marinade(s) => SvcAg::Marinade(s.accounts_to_update_svc()),
            SvcAg::SanctumSpl(s) => SvcAg::SanctumSpl(s.accounts_to_update_svc()),
            SvcAg::SanctumSplMulti(s) => SvcAg::SanctumSplMulti(s.accounts_to_update_svc()),
            SvcAg::Spl(s) => SvcAg::Spl(s.accounts_to_update_svc()),
            SvcAg::Wsol(s) => SvcAg::Wsol(s.accounts_to_update_svc()),
        }
    }
}

pub type UpdateSvcErr = SvcAg<
    LidoUpdateErr,
    MarinadeUpdateErr,
    SplUpdateErr,
    SplUpdateErr,
    SplUpdateErr,
    WsolUpdateErr,
>;

impl UpdateSvc for SvcAgStd {
    type InnerErr = UpdateSvcErr;

    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            SvcAg::Lido(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Lido)),
            SvcAg::Marinade(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Marinade)),
            SvcAg::SanctumSpl(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::SanctumSpl)),
            SvcAg::SanctumSplMulti(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::SanctumSplMulti)),
            SvcAg::Spl(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Spl)),
            SvcAg::Wsol(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Wsol)),
        }
    }
}

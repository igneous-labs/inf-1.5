use inf1_svc_ag_core::{map_variant_method, SvcAg};

use crate::SvcAgStd;

// Re-exports
pub use inf1_svc_inf_std::{InfExtPkIter, InfExtUpdateErr, InfUpdateErr, PkIter as InfPkIter};
pub use inf1_svc_lido_std::update::{LidoUpdateErr, PkIter as LidoPkIter};
pub use inf1_svc_marinade_std::update::{MarinadeUpdateErr, PkIter as MarinadePkIter};
pub use inf1_svc_sols_std::update::{PkIter as SolsPkIter, SolsUpdateErr};
pub use inf1_svc_spl_std::update::{PkIter as SplPkIter, SplUpdateErr};
pub use inf1_svc_std::update::*;
pub use inf1_svc_wsol_std::update::{PkIter as WsolPkIter, WsolUpdateErr};

pub type SvcPkIterAg = SvcAg<
    InfPkIter,
    InfExtPkIter,
    LidoPkIter,
    MarinadePkIter,
    SplPkIter,
    SplPkIter,
    SolsPkIter,
    SplPkIter,
    WsolPkIter,
>;

impl AccountsToUpdateSvc for SvcAgStd {
    type PkIter = SvcPkIterAg;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        map_variant_method!(self.0, accounts_to_update_svc())
    }
}

pub type UpdateSvcErr = SvcAg<
    InfUpdateErr,
    InfExtUpdateErr,
    LidoUpdateErr,
    MarinadeUpdateErr,
    SplUpdateErr,
    SplUpdateErr,
    SolsUpdateErr,
    SplUpdateErr,
    WsolUpdateErr,
>;

impl UpdateSvc for SvcAgStd {
    type InnerErr = UpdateSvcErr;

    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            SvcAg::Inf(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Inf)),
            SvcAg::InfExt(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::InfExt)),
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
            SvcAg::Sols(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Sols)),
            SvcAg::Spl(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Spl)),
            SvcAg::Wsol(s) => s
                .update_svc(update_map)
                .map_err(|e| e.map_inner(SvcAg::Wsol)),
        }
    }
}

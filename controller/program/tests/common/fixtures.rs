use inf1_std::inf1_pp_ag_std::{
    inf1_pp_flatslab_std::instructions::pricing::FlatSlabPpAccs,
    instructions::PriceLpTokensToMintAccsAg,
};
use inf1_svc_ag_core::{
    inf1_svc_marinade_core::instructions::sol_val_calc::MarinadeCalcAccs,
    inf1_svc_spl_core::instructions::sol_val_calc::SanctumSplMultiCalcAccs,
    instructions::SvcCalcAccsAg,
};
use inf1_test_utils::JUPSOL_POOL_ID;

pub fn jupsol_fixtures_svc_suf() -> SvcCalcAccsAg {
    SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs {
        stake_pool_addr: JUPSOL_POOL_ID.to_bytes(),
    })
}

pub fn msol_fixtures_svc_suf() -> SvcCalcAccsAg {
    SvcCalcAccsAg::Marinade(MarinadeCalcAccs)
}

pub fn flat_slab_pricing_fixture_suf() -> PriceLpTokensToMintAccsAg {
    PriceLpTokensToMintAccsAg::FlatSlab(FlatSlabPpAccs::MAINNET)
}

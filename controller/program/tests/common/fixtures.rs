use inf1_svc_ag_core::{
    inf1_svc_spl_core::instructions::sol_val_calc::SanctumSplMultiCalcAccs,
    instructions::SvcCalcAccsAg,
};
use inf1_test_utils::JUPSOL_POOL_ID;

/// TODO: replace with jupsol_fixture_svc_suf_accs()
pub fn jupsol_fixtures_svc_suf() -> SvcCalcAccsAg {
    SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs {
        stake_pool_addr: JUPSOL_POOL_ID.to_bytes(),
    })
}

// pub fn msol_fixtures_svc_suf() -> SvcCalcAccsAg {
//     SvcCalcAccsAg::Marinade(MarinadeCalcAccs)
// }

// pub fn flat_slab_pricing_fixture_suf() -> PriceLpTokensToMintAccsAg {
//     PriceLpTokensToMintAccsAg::FlatSlab(FlatSlabPpAccs::MAINNET)
// }

// pub fn flat_slab_pricing_lp_tokens_to_redeem_fixture_suf() -> PriceLpTokensToRedeemAccsAg {
//     PriceLpTokensToRedeemAccsAg::FlatSlab(FlatSlabPpAccs::MAINNET)
// }

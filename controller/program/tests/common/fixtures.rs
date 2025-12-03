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

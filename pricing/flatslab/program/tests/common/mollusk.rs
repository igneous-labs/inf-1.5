use agave_feature_set::FeatureSet;
use inf1_pp_flatslab_core::ID;
use mollusk_svm::{program::ProgramCache, Mollusk};
use solana_pubkey::Pubkey;

thread_local! {
    pub static MOLLUSK_NO_LOGS: Mollusk = mollusk_no_logs();
}

const PROG_NAME: &str = "inf1_pp_flatslab_program";

pub fn mollusk_no_logs() -> Mollusk {
    let mut m = new_mollusk_no_logs();
    m.add_program(
        &Pubkey::new_from_array(ID),
        PROG_NAME,
        &Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111"),
    );
    Mollusk::new(&Pubkey::new_from_array(ID), PROG_NAME)
}

fn new_mollusk_no_logs() -> Mollusk {
    // TODO: need to disable solana_logger but i dont wanna pull in that dependency

    // This is just the Default impl copy pasted with
    // - all features removed
    let feature_set = FeatureSet::all_enabled();
    let compute_budget = Default::default();
    let program_cache = ProgramCache::new(&feature_set, &compute_budget);
    Mollusk {
        compute_budget,
        feature_set,
        logger: Some(Default::default()),
        program_cache,
        ..Default::default()
    }
}

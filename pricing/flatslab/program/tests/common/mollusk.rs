use inf1_pp_flatslab_core::ID;
use mollusk_svm::Mollusk;
use solana_pubkey::Pubkey;

thread_local! {
    pub static MOLLUSK: Mollusk = Mollusk::new(&Pubkey::new_from_array(ID), PROG_NAME)
}

const PROG_NAME: &str = "inf1_pp_flatslab_program";

/// This needs to be ran outside the thread_local! static vars above
/// i.e. at the start of each proptest
/// or else it wont take effect
pub fn silence_mollusk_logs() {
    // set to level warn instead
    // of debug so we dont get overwhelmed by program logs
    // in our program proptests
    solana_logger::setup_with_default(
        "solana_rbpf::vm=warn,\
         solana_runtime::message_processor=warn,\
         solana_runtime::system_instruction_processor=warn",
    );
}

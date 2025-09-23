use inf1_pp_flatslab_core::ID;
use mollusk_svm::Mollusk;
use solana_pubkey::Pubkey;

const PROG_NAME: &str = "inf1_pp_flatslab_program";

thread_local! {
    pub static SVM: Mollusk = Mollusk::new(&Pubkey::new_from_array(ID), PROG_NAME)
}

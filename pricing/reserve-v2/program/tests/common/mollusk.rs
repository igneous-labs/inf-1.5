use std::cell::RefCell;

use inf1_pp_reserve_v2_core::keys::CONST_KEYS_OWNED;
use mollusk_svm::Mollusk;
use solana_pubkey::Pubkey;

const PROG_NAME: &str = "inf1_pp_reserve_v2_program";

thread_local! {
    pub static SVM: Mollusk = Mollusk::new(&Pubkey::new_from_array(*CONST_KEYS_OWNED.program()), PROG_NAME);

    pub static SVM_MUT: RefCell<Mollusk> =
        RefCell::new(Mollusk::new(&Pubkey::new_from_array(*CONST_KEYS_OWNED.program()), PROG_NAME));
}

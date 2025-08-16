use inf1_pp_flatslab_core::{accounts::Slab, typedefs::SlabEntryPacked};
use solana_pubkey::Pubkey;

mod remove_lst;
mod set_admin;
mod set_lst_fee;

pub fn assert_valid_slab(slab_acc_data: &[u8]) {
    let slab = Slab::of_acc_data(slab_acc_data).unwrap();
    assert!(slab.entries().0.is_sorted_by_key(|e| *e.mint()));
    // assert no duplicate entries
    for w in slab.entries().0.windows(2) {
        if w[0].mint() == w[1].mint() {
            panic!("duplicate {}", Pubkey::new_from_array(*w[0].mint()));
        }
    }
}

pub fn assert_slab_entry_on_slab(slab_acc_data: &[u8], expected: &SlabEntryPacked) {
    let slab_entries = Slab::of_acc_data(slab_acc_data).unwrap().entries();
    let actual = slab_entries.find_by_mint(expected.mint()).unwrap();
    assert_eq!(actual, expected);
}

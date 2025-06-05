use inf1_pp_flatfee_core::{pda::fee_account_seeds, ID};

use crate::pda::{create_raw_pda, find_pda};

pub(crate) fn find_fee_account_pda(lst_mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
    let (s1, s2) = fee_account_seeds(lst_mint);
    find_pda(&[s1, s2], &ID)
}

pub(crate) fn create_raw_fee_account_pda(lst_mint: &[u8; 32], bump: u8) -> [u8; 32] {
    let (s1, s2) = fee_account_seeds(lst_mint);
    // unwrap-safety: seeds are within range
    create_raw_pda([s1.as_slice(), s2.as_slice(), &[bump]], &ID).unwrap()
}

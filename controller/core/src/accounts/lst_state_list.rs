use crate::{accounts::packed_list::PackedListMut, typedefs::lst_state::LstStatePacked};

use super::packed_list::PackedList;

pub type LstStatePackedList<'a> = PackedList<'a, LstStatePacked>;

impl LstStatePackedList<'_> {
    #[inline]
    pub fn find_by_mint(&self, mint: &[u8; 32]) -> Option<&LstStatePacked> {
        self.0.iter().find(|s| s.mint == *mint)
    }
}

pub type LstStatePackedListMut<'a> = PackedListMut<'a, LstStatePacked>;

impl LstStatePackedListMut<'_> {
    #[inline]
    pub fn find_by_mint(&mut self, mint: &[u8; 32]) -> Option<&mut LstStatePacked> {
        self.0.iter_mut().find(|s| s.mint == *mint)
    }
}

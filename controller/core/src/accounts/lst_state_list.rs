use crate::{
    accounts::packed_list::PackedListMut,
    typedefs::lst_state::{LstState, LstStatePacked},
};

use super::packed_list::PackedList;

pub type LstStatePackedList<'a> = PackedList<'a, LstStatePacked>;

pub type LstStateList<'a> = PackedList<'a, LstState>;

impl LstStatePackedList<'_> {
    #[inline]
    pub fn find_by_mint(&self, mint: &[u8; 32]) -> Option<&LstStatePacked> {
        self.0.iter().find(|s| s.mint == *mint)
    }
}

pub type LstStatePackedListMut<'a> = PackedListMut<'a, LstStatePacked>;

pub type LstStateListMut<'a> = PackedListMut<'a, LstState>;

impl LstStatePackedListMut<'_> {
    #[inline]
    pub fn find_by_mint(&mut self, mint: &[u8; 32]) -> Option<&mut LstStatePacked> {
        self.0.iter_mut().find(|s| s.mint == *mint)
    }
}

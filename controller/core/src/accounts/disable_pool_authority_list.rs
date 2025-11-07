use crate::accounts::packed_list::{PackedList, PackedListMut};

pub type DisablePoolAuthorityList<'a> = PackedList<'a, [u8; 32]>;
pub type DisablePoolAuthorityListMut<'a> = PackedListMut<'a, [u8; 32]>;

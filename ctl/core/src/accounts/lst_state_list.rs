use crate::typedefs::lst_state::LstStatePacked;

use super::packed_list::PackedList;

pub type LstStatePackedList<'a> = PackedList<'a, LstStatePacked>;

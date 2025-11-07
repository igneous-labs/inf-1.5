use core::mem::size_of;

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RebalanceRecord {
    pub old_total_sol_value: u64,
    pub inp_lst_index: u32,
    pub padding: [u8; 4],
}
impl_cast_from_acc_data!(RebalanceRecord);
impl_cast_to_acc_data!(RebalanceRecord);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RebalanceRecordPacked {
    old_total_sol_value: [u8; 8],
    inp_lst_index: [u8; 4],
    padding: [u8; 4],
}
impl_cast_from_acc_data!(RebalanceRecordPacked, packed);
impl_cast_to_acc_data!(RebalanceRecordPacked, packed);

impl RebalanceRecordPacked {
    #[inline]
    pub const fn into_rebalance_record(self) -> RebalanceRecord {
        let Self {
            old_total_sol_value,
            inp_lst_index,
            padding,
        } = self;
        RebalanceRecord {
            old_total_sol_value: u64::from_le_bytes(old_total_sol_value),
            inp_lst_index: u32::from_le_bytes(inp_lst_index),
            padding,
        }
    }

    /// # Safety
    /// - `self` must be pointing to mem that has same align as `RebalanceRecord`.
    ///   This is true onchain for a RebalanceRecord account since account data
    ///   is always 8-byte aligned onchain.
    #[inline]
    pub const unsafe fn as_rebalance_record(&self) -> &RebalanceRecord {
        &*(self as *const Self).cast()
    }
}

impl From<RebalanceRecordPacked> for RebalanceRecord {
    #[inline]
    fn from(value: RebalanceRecordPacked) -> Self {
        value.into_rebalance_record()
    }
}

const _ASSERT_PACKED_UNPACKED_SIZES_EQ: () =
    assert!(size_of::<RebalanceRecord>() == size_of::<RebalanceRecordPacked>());

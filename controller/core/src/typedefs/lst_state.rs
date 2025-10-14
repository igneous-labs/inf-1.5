use core::mem::size_of;

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LstState {
    pub is_input_disabled: u8,
    pub pool_reserves_bump: u8,
    pub protocol_fee_accumulator_bump: u8,
    pub padding: [u8; 5],
    pub sol_value: u64,
    pub mint: [u8; 32],
    pub sol_value_calculator: [u8; 32],
}
impl_cast_from_acc_data!(LstState);
impl_cast_to_acc_data!(LstState);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LstStatePacked {
    pub(crate) is_input_disabled: u8,
    pub(crate) pool_reserves_bump: u8,
    pub(crate) protocol_fee_accumulator_bump: u8,
    pub(crate) padding: [u8; 5],
    pub(crate) sol_value: [u8; 8],
    pub(crate) mint: [u8; 32],
    pub(crate) sol_value_calculator: [u8; 32],
}
impl_cast_from_acc_data!(LstStatePacked, packed);
impl_cast_to_acc_data!(LstStatePacked, packed);

impl LstStatePacked {
    #[inline]
    pub const fn into_lst_state(self) -> LstState {
        let Self {
            is_input_disabled,
            pool_reserves_bump,
            protocol_fee_accumulator_bump,
            padding,
            sol_value,
            mint,
            sol_value_calculator,
        } = self;
        LstState {
            is_input_disabled,
            pool_reserves_bump,
            protocol_fee_accumulator_bump,
            padding,
            sol_value: u64::from_le_bytes(sol_value),
            mint,
            sol_value_calculator,
        }
    }

    /// # Safety
    /// - `self` must be pointing to mem that has same align as `LstState`.
    ///    This is true onchain for a LstStateList account since account data
    ///    is always 8-byte aligned onchain, and its a PackedList so offset of
    ///    first elem = 0.
    #[inline]
    pub const unsafe fn as_lst_state(&self) -> &LstState {
        &*(self as *const Self).cast()
    }

    /// # Safety
    /// - same rules as [`Self::as_lst_state`] apply
    #[inline]
    pub const unsafe fn as_lst_state_mut(&mut self) -> &mut LstState {
        &mut *(self as *mut Self).cast()
    }
}

impl From<LstStatePacked> for LstState {
    #[inline]
    fn from(value: LstStatePacked) -> Self {
        value.into_lst_state()
    }
}

const _ASSERT_PACKED_UNPACKED_SIZES_EQ: () =
    assert!(size_of::<LstState>() == size_of::<LstStatePacked>());

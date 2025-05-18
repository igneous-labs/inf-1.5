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
    is_input_disabled: u8,
    pool_reserves_bump: u8,
    protocol_fee_accumulator_bump: u8,
    padding: [u8; 5],
    sol_value: [u8; 8],
    mint: [u8; 32],
    sol_value_calculator: [u8; 32],
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
}

impl From<LstStatePacked> for LstState {
    #[inline]
    fn from(value: LstStatePacked) -> Self {
        value.into_lst_state()
    }
}

const _ASSERT_PACKED_UNPACKED_SIZES_EQ: () =
    assert!(size_of::<LstState>() == size_of::<LstStatePacked>());

use core::mem::size_of;

use super::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProgramState {
    pub manager: [u8; 32],
    pub lp_withdrawal_fee_bps: u16,
}
impl_cast_from_acc_data!(ProgramState);
impl_cast_to_acc_data!(ProgramState);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProgramStatePacked {
    manager: [u8; 32],
    lp_withdrawal_fee_bps: [u8; 2],
}
impl_cast_from_acc_data!(ProgramStatePacked, packed);
impl_cast_to_acc_data!(ProgramStatePacked, packed);

impl ProgramStatePacked {
    #[inline]
    pub const fn into_program_state(self) -> ProgramState {
        let Self {
            manager,
            lp_withdrawal_fee_bps,
        } = self;
        ProgramState {
            manager,
            lp_withdrawal_fee_bps: u16::from_le_bytes(lp_withdrawal_fee_bps),
        }
    }
}

impl From<ProgramStatePacked> for ProgramState {
    #[inline]
    fn from(value: ProgramStatePacked) -> Self {
        value.into_program_state()
    }
}

const _ASSERT_PACKED_UNPACKED_SIZES_EQ: () =
    assert!(size_of::<ProgramState>() == size_of::<ProgramStatePacked>());

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct State {
    pub manager: [u8; 32],
    pub last_upgrade_slot: u64,
}
impl_cast_from_acc_data!(State);
impl_cast_to_acc_data!(State);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct StatePacked {
    manager: [u8; 32],
    last_upgrade_slot: [u8; 8],
}
impl_cast_from_acc_data!(StatePacked, packed);
impl_cast_to_acc_data!(StatePacked, packed);

impl StatePacked {
    #[inline]
    pub const fn into_state(self) -> State {
        let Self {
            manager,
            last_upgrade_slot,
        } = self;
        State {
            manager,
            last_upgrade_slot: u64::from_le_bytes(last_upgrade_slot),
        }
    }
}

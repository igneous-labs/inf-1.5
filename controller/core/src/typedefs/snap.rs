use generic_array_struct::generic_array_struct;

use crate::typedefs::update_dir::UpdateDir;

/// A state snapshot across time
#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Snap<T> {
    pub old: T,
    pub new: T,
}

impl<T: Copy> Snap<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; SNAP_LEN])
    }
}

pub type SnapU64 = Snap<u64>;

impl SnapU64 {
    /// Returns the change from old to new
    #[inline]
    pub const fn delta(&self) -> (UpdateDir, u64) {
        // unchecked-arith: bounds checked here
        if *self.new() >= *self.old() {
            (UpdateDir::Inc, *self.new() - *self.old())
        } else {
            (UpdateDir::Dec, *self.old() - *self.new())
        }
    }
}

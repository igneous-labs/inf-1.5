use generic_array_struct::generic_array_struct;

/// A state snapshot across time
#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Snap<T> {
    pub old: T,
    pub new: T,
}

pub type SnapU64 = Snap<u64>;

/// An internally defined newtype ZST wrapper over the unit type.
///
/// For avoiding issues with orphan rules with traits
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Empty(pub ());

impl Empty {
    pub const SELF: Self = Self(());
}

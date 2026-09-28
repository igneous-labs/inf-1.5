use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use crate::typedefs::Vers;

pub type PoolFlags = VersOnlyFlags;
pub type ProtocolFlags = VersOnlyFlags;
pub type HoldingFlags = VersOnlyFlags;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct VersOnlyFlags(u8);

impl VersOnlyFlags {
    pub const BITS_USED: usize = 2;
    pub const BITS_USED_AND_MASK: u8 = (1 << Self::BITS_USED) - 1;

    pub const VERS_AND_MASK: u8 = 0b_0000_0011;

    pub const V1: Self = Self(Vers::V1.into_u8());

    #[inline]
    pub const fn vers(&self) -> Vers {
        // safety: self is correct at construction
        unsafe { Vers::try_from_u8(self.0 & Self::VERS_AND_MASK).unwrap_unchecked() }
    }

    /// # Safety
    /// - arg must be of a valid bitpattern
    #[inline]
    pub const unsafe fn of_ref(flag: &u8) -> &Self {
        let this = core::ptr::from_ref(flag).cast();
        // safety: repr(transparent)
        unsafe { &*this }
    }

    /// Returns `None` if arg not of valid bitpattern
    #[inline]
    pub const fn try_of_ref(flag: &u8) -> Option<&Self> {
        if flag.leading_zeros() < Self::BITS_USED_AND_MASK.leading_zeros() {
            return None;
        }
        if Vers::try_from_u8(*flag & Self::VERS_AND_MASK).is_none() {
            return None;
        }
        Some(unsafe { Self::of_ref(flag) })
    }

    #[inline]
    pub const fn as_byte(&self) -> &u8 {
        &self.0
    }
}

impl From<VersOnlyFlags> for u8 {
    #[inline]
    fn from(v: VersOnlyFlags) -> Self {
        *v.as_byte()
    }
}

impl Default for VersOnlyFlags {
    #[inline]
    fn default() -> Self {
        Self::V1
    }
}

impl VersOnlyFlags {
    #[inline]
    const fn used_bits_only(&self) -> u8 {
        self.0 & Self::BITS_USED_AND_MASK
    }

    #[inline]
    pub const fn const_cmp(&self, other: &Self) -> Ordering {
        let this = self.used_bits_only();
        let other = other.used_bits_only();
        if this == other {
            Ordering::Equal
        } else if this < other {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl PartialEq for VersOnlyFlags {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        matches!(self.const_cmp(other), Ordering::Equal)
    }
}

impl Eq for VersOnlyFlags {}

impl Ord for VersOnlyFlags {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.const_cmp(other)
    }
}

impl PartialOrd for VersOnlyFlags {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// to maintain
/// `k1 == k2 -> hash(k1) == hash(k2)`
/// invariant
impl Hash for VersOnlyFlags {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.used_bits_only().hash(state);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for VersOnlyFlags {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        vers_only_flags_serde::serialize(self, s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VersOnlyFlags {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        vers_only_flags_serde::deserialize(d)
    }
}

/// serde(with) compatible module
#[cfg(feature = "serde")]
pub mod vers_only_flags_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    pub fn serialize<S: Serializer>(v: &VersOnlyFlags, s: S) -> Result<S::Ok, S::Error> {
        v.vers().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<VersOnlyFlags, D::Error> {
        Vers::deserialize(d).map(
            // unwrap-safety: Vers::deserialize ensures valid bitpattern
            |f| *VersOnlyFlags::try_of_ref(&f.into_u8()).unwrap(),
        )
    }
}

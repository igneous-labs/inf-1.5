use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

/// A single-byte backed boolean value where
/// `0` represents `false` and any other bit pattern
/// represents `true`
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct U8Bool(pub u8);

impl U8Bool {
    #[inline]
    pub const fn into_bool(&self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub const fn from_bool(b: bool) -> Self {
        match b {
            true => Self(1),
            false => Self(0),
        }
    }

    #[inline]
    pub const fn of_u8(b: &u8) -> &Self {
        let this = core::ptr::from_ref(b).cast();
        // safety: repr(transparent)
        unsafe { &*this }
    }

    #[inline]
    pub const fn of_u8_mut(b: &mut u8) -> &mut Self {
        let this = core::ptr::from_mut(b).cast();
        // safety: repr(transparent)
        unsafe { &mut *this }
    }
}

impl From<bool> for U8Bool {
    #[inline]
    fn from(v: bool) -> Self {
        Self::from_bool(v)
    }
}

impl From<U8Bool> for bool {
    #[inline]
    fn from(v: U8Bool) -> Self {
        v.into_bool()
    }
}

impl PartialEq for U8Bool {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.into_bool().eq(&other.into_bool())
    }
}

impl Eq for U8Bool {}

impl Ord for U8Bool {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_bool().cmp(&other.into_bool())
    }
}

impl PartialOrd for U8Bool {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// to maintain
/// `k1 == k2 -> hash(k1) == hash(k2)`
/// invariant
impl Hash for U8Bool {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.into_bool().hash(state);
    }
}

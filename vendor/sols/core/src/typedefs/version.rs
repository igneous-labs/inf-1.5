#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Vers {
    V1,
    V2,
    V3,
}

impl Vers {
    /// Illegal bit patterns (anything other than 01, 10, 11)
    /// returns `None`
    #[inline]
    pub const fn try_from_u8(v: u8) -> Option<Self> {
        match v {
            0b_01 => Some(Vers::V1),
            0b_10 => Some(Vers::V2),
            0b_11 => Some(Vers::V3),
            _ => None,
        }
    }

    #[inline]
    pub const fn into_u8(self) -> u8 {
        match self {
            Vers::V1 => 0b_01,
            Vers::V2 => 0b_10,
            Vers::V3 => 0b_11,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Vers {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        vers_serde::serialize(self, s)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Vers {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        vers_serde::deserialize(d)
    }
}

/// serde(with) compatible module
#[cfg(feature = "serde")]
pub mod vers_serde {
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    use super::*;

    pub fn serialize<S: Serializer>(v: &Vers, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(v.into_u8())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vers, D::Error> {
        u8::deserialize(d)
            .and_then(|u| Vers::try_from_u8(u).ok_or_else(|| Error::custom("invalid bitpattern")))
    }
}

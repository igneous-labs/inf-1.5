use generic_array_struct::generic_array_struct;
use inf1_svc_core::traits::SolValCalcAccs;

/// This program has no additional accounts suffix
#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxSufAccs<T> {}

impl<T> IxSufAccs<T> {
    #[inline]
    pub const fn new() -> Self {
        Self([])
    }
}

pub type IxSufKeys<'a> = IxSufAccs<&'a [u8; 32]>;

pub type IxSufKeysOwned = IxSufAccs<[u8; 32]>;

pub type IxSufAccFlags = IxSufAccs<bool>;

impl<T> AsRef<[T]> for IxSufAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WsolCalcAccs;

impl WsolCalcAccs {
    #[inline]
    pub const fn svc_suf_keys_owned(&self) -> IxSufKeysOwned {
        IxSufAccs::new()
    }

    #[inline]
    pub const fn svc_suf_is_writer(&self) -> IxSufAccFlags {
        IxSufAccs::new()
    }

    #[inline]
    pub const fn svc_suf_is_signer(&self) -> IxSufAccFlags {
        IxSufAccs::new()
    }
}

impl SolValCalcAccs for WsolCalcAccs {
    type KeysOwned = IxSufKeysOwned;

    type AccFlags = IxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.svc_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.svc_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.svc_suf_is_signer()
    }
}

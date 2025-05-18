use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::impl_asref;

pub mod exact_in;
pub mod exact_out;

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxSufAccs<T> {
    /// Fee account PDA of the input LST mint
    pub input_fee: T,

    /// Fee account PDA of the output LST mint
    pub output_fee: T,
}

impl<T: Copy> IxSufAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; IX_SUF_ACCS_LEN])
    }
}

pub type IxSufKeys<'a> = IxSufAccs<&'a [u8; 32]>;

pub type IxSufKeysOwned = IxSufAccs<[u8; 32]>;

pub type IxSufAccFlags = IxSufAccs<bool>;

pub const IX_SUF_IS_WRITER: IxSufAccFlags = IxSufAccFlags::memset(false);

pub const IX_SUF_IS_SIGNER: IxSufAccFlags = IxSufAccFlags::memset(false);

impl_asref!(IxSufAccs<T>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FlatFeePriceAccs(pub IxSufKeysOwned);

impl FlatFeePriceAccs {
    #[inline]
    pub const fn pp_price_suf_keys_owned(&self) -> IxSufKeysOwned {
        self.0
    }

    #[inline]
    pub const fn pp_price_suf_is_writer(&self) -> IxSufAccFlags {
        IX_SUF_IS_WRITER
    }

    #[inline]
    pub const fn pp_price_suf_is_signer(&self) -> IxSufAccFlags {
        IX_SUF_IS_SIGNER
    }
}

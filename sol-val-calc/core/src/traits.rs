use core::ops::{Deref, RangeInclusive};

pub trait SolValCalc {
    type Error;

    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
}

/// Blanket for refs
impl<R, T: SolValCalc> SolValCalc for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.deref().lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.deref().sol_to_lst(lamports_amount)
    }
}

/// Suffix account meta slices returned by the 3 methods must all have the same length.
///
/// Append the suffix to the prefixes [`crate::instructions::IxPreKeys`] to create
/// the account inputs of a full interface instruction
pub trait SolValCalcAccs {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;

    fn suf_is_writer(&self) -> Self::AccFlags;

    fn suf_is_signer(&self) -> Self::AccFlags;
}

/// Blanket for refs
impl<R, T: SolValCalcAccs> SolValCalcAccs for R
where
    R: Deref<Target = T>,
{
    type KeysOwned = T::KeysOwned;

    type AccFlags = T::AccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.deref().suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.deref().suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.deref().suf_is_signer()
    }
}

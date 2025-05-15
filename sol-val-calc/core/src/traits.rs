use core::ops::RangeInclusive;

pub trait SolValCalc {
    type Error;

    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
}

/// Suffix account meta slices returned by the 3 methods must all have the same length.
///
/// Append the suffix to the prefixes [`crate::instructions::IxPreKeys`] to create
/// the account inputs of a full interface instruction
pub trait SolValCalcProgram {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;

    fn suf_is_writer(&self) -> Self::AccFlags;

    fn suf_is_signer(&self) -> Self::AccFlags;
}

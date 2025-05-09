use core::ops::RangeInclusive;

pub trait SolValCalc {
    type Error;

    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
}

/// Implement [`SolValCalc`] methods as inherent methods so that they're available to call
/// even if the trait is not in scope
#[macro_export]
macro_rules! inh_sol_val_calc {
    () => {
        #[inline]
        pub fn svc_lst_to_sol(
            &self,
            lst_amount: u64,
        ) -> Result<core::ops::RangeInclusive<u64>, <Self as $crate::traits::SolValCalc::Error>> {
            <Self as $crate::traits::SolValCalc>::lst_to_sol(self, lst_amount)
        }

        #[inline]
        pub fn svc_sol_to_lst(
            &self,
            lamports_amount: u64,
        ) -> Result<core::ops::RangeInclusive<u64>, <Self as $crate::traits::SolValCalc::Error>> {
            <Self as $crate::traits::SolValCalc>::sol_to_lst(self, lamports_amount)
        }
    };
}

/// Suffix account meta slices returned by the 3 methods must all have the same length.
pub trait SolValCalcProgram {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;

    fn suf_is_writer(&self) -> Self::AccFlags;

    fn suf_is_signer(&self) -> Self::AccFlags;
}

/// Implement [`SolValCalcProgram`] methods as inherent methods so that they're available to call
/// even if the trait is not in scope
#[macro_export]
macro_rules! inh_sol_val_calc_prog {
    () => {
        #[inline]
        pub fn svcp_suf_keys_owned(
            &self,
        ) -> <Self as $crate::traits::SolValCalcProgram>::KeysOwned {
            <Self as $crate::traits::SolValCalcProgram>::suf_keys_owned(self)
        }

        #[inline]
        pub fn svcp_suf_is_writer(&self) -> <Self as $crate::traits::SolValCalcProgram>::AccFlags {
            <Self as $crate::traits::SolValCalcProgram>::suf_is_writer(self)
        }

        #[inline]
        pub fn svcp_suf_is_signer(&self) -> <Self as $crate::traits::SolValCalcProgram>::AccFlags {
            <Self as $crate::traits::SolValCalcProgram>::suf_is_signer(self)
        }
    };
}

use core::ops::Deref;

use crate::pair::Pair;

// Quoting

pub trait PriceExactInCol {
    type Error: core::error::Error;
    type PriceExactIn: crate::traits::main::PriceExactIn;

    fn price_exact_in_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceExactInCol> PriceExactInCol for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;
    type PriceExactIn = T::PriceExactIn;

    #[inline]
    fn price_exact_in_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error> {
        self.deref().price_exact_in_for(mints)
    }
}

pub trait PriceExactOutCol {
    type Error: core::error::Error;
    type PriceExactIn: crate::traits::main::PriceExactOut;

    fn price_exact_out_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceExactOutCol> PriceExactOutCol for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;
    type PriceExactIn = T::PriceExactIn;

    #[inline]
    fn price_exact_out_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error> {
        self.deref().price_exact_out_for(mints)
    }
}

// Accounts

pub trait PriceExactInAccsCol {
    type Error: core::error::Error;
    type PriceExactInAccs: crate::traits::main::PriceExactInAccs;

    fn price_exact_in_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactInAccs, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceExactInAccsCol> PriceExactInAccsCol for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;
    type PriceExactInAccs = T::PriceExactInAccs;

    #[inline]
    fn price_exact_in_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactInAccs, Self::Error> {
        self.deref().price_exact_in_accs_for(mints)
    }
}

pub trait PriceExactOutAccsCol {
    type Error: core::error::Error;
    type PriceExactOutAccs: crate::traits::main::PriceExactOutAccs;

    fn price_exact_out_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOutAccs, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceExactOutAccsCol> PriceExactOutAccsCol for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;
    type PriceExactOutAccs = T::PriceExactOutAccs;

    #[inline]
    fn price_exact_out_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOutAccs, Self::Error> {
        self.deref().price_exact_out_accs_for(mints)
    }
}

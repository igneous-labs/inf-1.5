use std::{convert::Infallible, error::Error, fmt::Display};

use inf1_pp_flatslab_core::{
    instructions::pricing::FlatSlabPpAccs, pricing::FlatSlabSwapPricing, typedefs::MintNotFoundErr,
};
use inf1_pp_std::{
    pair::Pair,
    traits::collection::{
        PriceExactInAccsCol, PriceExactInCol, PriceExactOutAccsCol, PriceExactOutCol,
    },
};

use crate::FlatSlabPricing;

pub mod deprecated;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatSlabPricingColErr {
    MintNotFound(MintNotFoundErr),
}

impl Display for FlatSlabPricingColErr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MintNotFound(e) => Display::fmt(&e, f),
        }
    }
}

impl Error for FlatSlabPricingColErr {}

impl From<MintNotFoundErr> for FlatSlabPricingColErr {
    #[inline]
    fn from(e: MintNotFoundErr) -> Self {
        Self::MintNotFound(e)
    }
}

impl From<Infallible> for FlatSlabPricingColErr {
    #[inline]
    fn from(_e: Infallible) -> Self {
        unreachable!()
    }
}

// Quoting

impl FlatSlabPricing {
    /// Returns first missing mint if either `FeeAccount`s are missing
    #[inline]
    pub fn flat_slab_swap_pricing_for(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<FlatSlabSwapPricing, MintNotFoundErr> {
        let entries = self.entries();
        let Pair { inp, out } = pair.try_map(|mint| entries.find_by_mint(mint))?;
        Ok(FlatSlabSwapPricing {
            inp_fee_nanos: inp.inp_fee_nanos(),
            out_fee_nanos: out.out_fee_nanos(),
        })
    }
}

impl PriceExactInCol for FlatSlabPricing {
    type Error = FlatSlabPricingColErr;
    type PriceExactIn = FlatSlabSwapPricing;

    #[inline]
    fn price_exact_in_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error> {
        self.flat_slab_swap_pricing_for(mints).map_err(Into::into)
    }
}

impl PriceExactOutCol for FlatSlabPricing {
    type Error = FlatSlabPricingColErr;
    type PriceExactOut = FlatSlabSwapPricing;

    #[inline]
    fn price_exact_out_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOut, Self::Error> {
        self.flat_slab_swap_pricing_for(mints).map_err(Into::into)
    }
}

// Accounts

impl FlatSlabPricing {
    #[inline]
    pub const fn flat_slab_pp_accs(&self) -> FlatSlabPpAccs {
        FlatSlabPpAccs::MAINNET
    }
}

impl PriceExactInAccsCol for FlatSlabPricing {
    type Error = Infallible;

    type PriceExactInAccs = FlatSlabPpAccs;

    #[inline]
    fn price_exact_in_accs_for(
        &self,
        _mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactInAccs, Self::Error> {
        Ok(self.flat_slab_pp_accs())
    }
}

impl PriceExactOutAccsCol for FlatSlabPricing {
    type Error = Infallible;

    type PriceExactOutAccs = FlatSlabPpAccs;

    #[inline]
    fn price_exact_out_accs_for(
        &self,
        _mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOutAccs, Self::Error> {
        Ok(self.flat_slab_pp_accs())
    }
}

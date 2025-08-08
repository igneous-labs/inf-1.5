use inf1_pp_ag_core::{instructions::PriceExactInAccsAg, pricing::PriceExactInAg, PricingAg};
use inf1_pp_std::{
    pair::Pair,
    traits::collection::{PriceExactInAccsCol, PriceExactInCol},
};

use crate::{PricingProgAg, PricingProgAgErr};

impl<F, C> PriceExactInCol for PricingProgAg<F, C> {
    type Error = PricingProgAgErr;
    type PriceExactIn = PriceExactInAg;

    #[inline]
    fn price_exact_in_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => p
                .price_exact_in_for(mints)
                .map(PricingAg::FlatFee)
                .map_err(PricingAg::FlatFee),
        }
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > PriceExactInAccsCol for PricingProgAg<F, C>
{
    type Error = PricingProgAgErr;

    type PriceExactInAccs = PriceExactInAccsAg;

    #[inline]
    fn price_exact_in_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactInAccs, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => Ok(p
                .price_exact_in_accs_for(mints)
                .map(PricingAg::FlatFee)
                .unwrap()), // unwrap-safety: infallible
        }
    }
}

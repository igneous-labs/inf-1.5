use inf1_pp_ag_core::{instructions::PriceExactInAccsAg, pricing::PriceExactInAg, PricingAg};
use inf1_pp_std::{
    pair::Pair,
    traits::collection::{PriceExactInAccsCol, PriceExactInCol},
};

use crate::{internal_utils::map_variant_method_fallible, PricingProgAg, PricingProgAgErr};

impl<F, C> PriceExactInCol for PricingProgAg<F, C> {
    type Error = PricingProgAgErr;
    type PriceExactIn = PriceExactInAg;

    #[inline]
    fn price_exact_in_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error> {
        map_variant_method_fallible!(&self.0, price_exact_in_for(mints))
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
        map_variant_method_fallible!(&self.0, price_exact_in_accs_for(mints))
    }
}

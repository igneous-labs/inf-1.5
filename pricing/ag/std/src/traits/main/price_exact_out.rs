use inf1_pp_ag_core::{instructions::PriceExactOutAccsAg, pricing::PriceExactOutAg, PricingAg};
use inf1_pp_std::{
    pair::Pair,
    traits::collection::{PriceExactOutAccsCol, PriceExactOutCol},
};

use crate::{internal_utils::map_variant_method_fallible, PricingProgAg, PricingProgAgErr};

impl<F, C> PriceExactOutCol for PricingProgAg<F, C> {
    type Error = PricingProgAgErr;
    type PriceExactOut = PriceExactOutAg;

    #[inline]
    fn price_exact_out_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOut, Self::Error> {
        map_variant_method_fallible!(&self.0, price_exact_out_for(mints))
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > PriceExactOutAccsCol for PricingProgAg<F, C>
{
    type Error = PricingProgAgErr;

    type PriceExactOutAccs = PriceExactOutAccsAg;

    #[inline]
    fn price_exact_out_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOutAccs, Self::Error> {
        map_variant_method_fallible!(&self.0, price_exact_out_accs_for(mints))
    }
}

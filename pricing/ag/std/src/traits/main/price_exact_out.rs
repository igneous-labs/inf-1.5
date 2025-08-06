use inf1_pp_ag_core::{instructions::PriceExactOutAccsAg, pricing::PriceExactOutAg, PricingAg};
use inf1_pp_std::{
    pair::Pair,
    traits::collection::{PriceExactOutAccsCol, PriceExactOutCol},
};

use crate::{PricingProgAg, PricingProgAgErr, PricingProgAgInfallibleErr};

impl<F, C> PriceExactOutCol for PricingProgAg<F, C> {
    type Error = PricingProgAgErr;
    type PriceExactOut = PriceExactOutAg;

    #[inline]
    fn price_exact_out_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOut, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => p
                .price_exact_out_for(mints)
                .map(PricingAg::FlatFee)
                .map_err(PricingAg::FlatFee),
        }
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > PriceExactOutAccsCol for PricingProgAg<F, C>
{
    type Error = PricingProgAgInfallibleErr;

    type PriceExactOutAccs = PriceExactOutAccsAg;

    #[inline]
    fn price_exact_out_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOutAccs, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => p
                .price_exact_out_accs_for(mints)
                .map(PricingAg::FlatFee)
                .map_err(PricingAg::FlatFee),
        }
    }
}

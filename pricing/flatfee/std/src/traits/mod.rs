//! Implementation of collection traits on [`crate::FlatFeePricing`]

use core::{error::Error, fmt::Display};
use std::convert::Infallible;

use inf1_pp_core::{
    pair::Pair,
    traits::collection::{
        PriceExactInAccsCol, PriceExactInCol, PriceExactOutAccsCol, PriceExactOutCol,
    },
};
use inf1_pp_flatfee_core::{
    accounts::fee::FeeAccount,
    instructions::pricing::price::{FlatFeePriceAccs, NewIxSufAccsBuilder},
    pricing::price::FlatFeeSwapPricing,
};

use crate::FlatFeePricing;

pub mod deprecated;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlatFeePricingColErr {
    AccMissing,
}

impl Display for FlatFeePricingColErr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl Error for FlatFeePricingColErr {}

// Quoting

impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub fn flat_fee_swap_pricing_for(
        &self,
        Pair { inp, out }: &Pair<&[u8; 32]>,
    ) -> Option<FlatFeeSwapPricing> {
        let [Some(FeeAccount { input_fee_bps, .. }), Some(FeeAccount { output_fee_bps, .. })] =
            [inp, out].map(|mint| self.fee_account(*mint))
        else {
            return None;
        };
        Some(FlatFeeSwapPricing {
            input_fee_bps: *input_fee_bps,
            output_fee_bps: *output_fee_bps,
        })
    }
}

impl<F, C> PriceExactInCol for FlatFeePricing<F, C> {
    type Error = FlatFeePricingColErr;
    type PriceExactIn = FlatFeeSwapPricing;

    #[inline]
    fn price_exact_in_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error> {
        self.flat_fee_swap_pricing_for(mints)
            .ok_or(FlatFeePricingColErr::AccMissing)
    }
}

impl<F, C> PriceExactOutCol for FlatFeePricing<F, C> {
    type Error = FlatFeePricingColErr;
    type PriceExactOut = FlatFeeSwapPricing;

    #[inline]
    fn price_exact_out_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOut, Self::Error> {
        self.flat_fee_swap_pricing_for(mints)
            .ok_or(FlatFeePricingColErr::AccMissing)
    }
}

// Accounts

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > FlatFeePricing<F, C>
{
    #[inline]
    pub fn flat_fee_price_accs_for(&self, Pair { inp, out }: &Pair<&[u8; 32]>) -> FlatFeePriceAccs {
        FlatFeePriceAccs(
            NewIxSufAccsBuilder::start()
                .with_input_fee(self.fee_account_pda(inp))
                .with_output_fee(self.fee_account_pda(out))
                .build(),
        )
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > PriceExactInAccsCol for FlatFeePricing<F, C>
{
    type Error = Infallible;

    type PriceExactInAccs = FlatFeePriceAccs;

    #[inline]
    fn price_exact_in_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactInAccs, Self::Error> {
        Ok(self.flat_fee_price_accs_for(mints))
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > PriceExactOutAccsCol for FlatFeePricing<F, C>
{
    type Error = Infallible;

    type PriceExactOutAccs = FlatFeePriceAccs;

    #[inline]
    fn price_exact_out_accs_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOutAccs, Self::Error> {
        Ok(self.flat_fee_price_accs_for(mints))
    }
}

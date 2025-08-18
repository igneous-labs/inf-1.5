//! Implementation of collection traits on [`crate::FlatFeePricing`]

use core::{error::Error, fmt::Display};
use std::convert::Infallible;

use inf1_pp_flatfee_core::{
    accounts::fee::FeeAccount,
    instructions::pricing::price::{FlatFeePriceAccs, NewIxSufAccsBuilder},
    pricing::price::FlatFeeSwapPricing,
};
use inf1_pp_std::{
    pair::Pair,
    traits::collection::{
        PriceExactInAccsCol, PriceExactInCol, PriceExactOutAccsCol, PriceExactOutCol,
    },
};

use crate::FlatFeePricing;

pub mod deprecated;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlatFeePricingColErr {
    FeeAccountMissing { mint: [u8; 32] },
    ProgramStateMissing,
}

impl Display for FlatFeePricingColErr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:#?}")
    }
}

impl Error for FlatFeePricingColErr {}

impl From<Infallible> for FlatFeePricingColErr {
    #[inline]
    fn from(_e: Infallible) -> Self {
        unreachable!()
    }
}

// Quoting

impl<F, C> FlatFeePricing<F, C> {
    /// Returns first missing mint if either `FeeAccount`s are missing
    #[inline]
    pub fn flat_fee_swap_pricing_for<'a>(
        &self,
        pair: &Pair<&'a [u8; 32]>,
    ) -> Result<FlatFeeSwapPricing, &'a [u8; 32]> {
        let Pair {
            inp: FeeAccount { input_fee_bps, .. },
            out: FeeAccount { output_fee_bps, .. },
        } = pair.try_map(|mint| self.fee_account(mint).ok_or(mint))?;
        Ok(FlatFeeSwapPricing {
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
            .map_err(|m| FlatFeePricingColErr::FeeAccountMissing { mint: *m })
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
            .map_err(|m| FlatFeePricingColErr::FeeAccountMissing { mint: *m })
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

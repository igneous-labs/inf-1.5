use core::{convert::Infallible, error::Error, fmt::Display};

use inf1_pp_core::{
    instructions::deprecated::lp::mint::PriceLpTokensToMintIxArgs,
    traits::deprecated::PriceLpTokensToMint,
};
use inf1_pp_flatfee_core::pricing::lp::FlatFeeMintLpPricing;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PriceMintLpAg {
    FlatFee(FlatFeeMintLpPricing),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriceMintLpAgErr {
    FlatFee(Infallible),
}

impl Display for PriceMintLpAgErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FlatFee(e) => e.fmt(f),
        }
    }
}

impl Error for PriceMintLpAgErr {}

impl PriceLpTokensToMint for PriceMintLpAg {
    type Error = PriceMintLpAgErr;

    fn price_lp_tokens_to_mint(
        &self,
        input: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        match self {
            Self::FlatFee(p) => p
                .price_lp_tokens_to_mint(input)
                .map_err(PriceMintLpAgErr::FlatFee),
        }
    }
}

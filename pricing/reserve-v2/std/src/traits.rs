use std::convert::Infallible;

use inf1_pp_reserve_v2_core::{
    instructions::pricing::ReserveV2PpAccs, keys::CONST_KEYS_OWNED, pricing::ReserveV2SwapPricing,
};
use inf1_pp_std::{
    pair::Pair,
    traits::collection::{
        PriceExactInAccsCol, PriceExactInCol, PriceExactOutAccsCol, PriceExactOutCol,
    },
};

use crate::{ReserveV2Pricing, ReserveV2PricingColErr};

// Quoting

impl PriceExactInCol for ReserveV2Pricing {
    type Error = ReserveV2PricingColErr;
    type PriceExactIn = ReserveV2SwapPricing;

    #[inline]
    fn price_exact_in_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactIn, Self::Error> {
        self.swap_pricing_for(mints)
    }
}

impl PriceExactOutCol for ReserveV2Pricing {
    type Error = ReserveV2PricingColErr;
    type PriceExactOut = ReserveV2SwapPricing;

    #[inline]
    fn price_exact_out_for(
        &self,
        mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOut, Self::Error> {
        self.swap_pricing_for(mints)
    }
}

// Accounts. The suffix is the same 3 const PDAs regardless of the pair; only
// the *quoting* depends on the pricing state / pool snapshot.

impl PriceExactInAccsCol for ReserveV2Pricing {
    type Error = Infallible;
    type PriceExactInAccs = ReserveV2PpAccs;

    #[inline]
    fn price_exact_in_accs_for(
        &self,
        _mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactInAccs, Self::Error> {
        Ok(ReserveV2PpAccs::MAINNET)
    }
}

impl PriceExactOutAccsCol for ReserveV2Pricing {
    type Error = Infallible;
    type PriceExactOutAccs = ReserveV2PpAccs;

    #[inline]
    fn price_exact_out_accs_for(
        &self,
        _mints: &Pair<&[u8; 32]>,
    ) -> Result<Self::PriceExactOutAccs, Self::Error> {
        Ok(ReserveV2PpAccs::MAINNET)
    }
}

// Deprecated LP interface. The reserve-v2 pricing program rejects these
// instructions, but the aggregate still has to name a type, so the quote goes
// through the same route resolution and fails at the trait method.

pub mod deprecated {
    #![allow(deprecated)]

    use super::*;
    use inf1_pp_std::traits::deprecated::{
        PriceLpTokensToMintAccsCol, PriceLpTokensToMintCol, PriceLpTokensToRedeemAccsCol,
        PriceLpTokensToRedeemCol,
    };

    impl PriceLpTokensToMintCol for ReserveV2Pricing {
        type Error = ReserveV2PricingColErr;
        type PriceLpTokensToMint = ReserveV2SwapPricing;

        #[inline]
        fn price_lp_tokens_to_mint_for(
            &self,
            inp_mint: &[u8; 32],
        ) -> Result<Self::PriceLpTokensToMint, Self::Error> {
            self.swap_pricing_for(&Pair {
                inp: inp_mint,
                out: CONST_KEYS_OWNED.lp_mint(),
            })
        }
    }

    impl PriceLpTokensToRedeemCol for ReserveV2Pricing {
        type Error = ReserveV2PricingColErr;
        type PriceLpTokensToRedeem = ReserveV2SwapPricing;

        #[inline]
        fn price_lp_tokens_to_redeem_for(
            &self,
            out_mint: &[u8; 32],
        ) -> Result<Self::PriceLpTokensToRedeem, Self::Error> {
            self.swap_pricing_for(&Pair {
                inp: CONST_KEYS_OWNED.lp_mint(),
                out: out_mint,
            })
        }
    }

    impl PriceLpTokensToMintAccsCol for ReserveV2Pricing {
        type Error = Infallible;
        type PriceLpTokensToMintAccs = ReserveV2PpAccs;

        #[inline]
        fn price_lp_tokens_to_mint_accs_for(
            &self,
            _inp_mint: &[u8; 32],
        ) -> Result<Self::PriceLpTokensToMintAccs, Self::Error> {
            Ok(ReserveV2PpAccs::MAINNET)
        }
    }

    impl PriceLpTokensToRedeemAccsCol for ReserveV2Pricing {
        type Error = Infallible;
        type PriceLpTokensToRedeemAccs = ReserveV2PpAccs;

        #[inline]
        fn price_lp_tokens_to_redeem_accs_for(
            &self,
            _out_mint: &[u8; 32],
        ) -> Result<Self::PriceLpTokensToRedeemAccs, Self::Error> {
            Ok(ReserveV2PpAccs::MAINNET)
        }
    }
}

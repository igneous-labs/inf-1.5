use inf1_pp_core::traits::deprecated::PriceLpTokensToRedeemAccs;
use inf1_pp_flatfee_core::instructions::pricing::lp::redeem::FlatFeeRedeemLpAccs;

use crate::PricingAg;

pub type PriceLpTokensToRedeemAccsAg = PricingAg<FlatFeeRedeemLpAccs>;

type FlatFeeKeysOwned = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::AccFlags;

impl PriceLpTokensToRedeemAccs for PriceLpTokensToRedeemAccsAg {
    type KeysOwned = PricingAg<FlatFeeKeysOwned>;
    type AccFlags = PricingAg<FlatFeeAccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_keys_owned()),
        }
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_writer()),
        }
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_signer()),
        }
    }
}

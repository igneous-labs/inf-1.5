use inf1_pp_core::traits::deprecated::PriceLpTokensToRedeemAccs;
use inf1_pp_flatfee_core::instructions::pricing::lp::redeem::FlatFeeRedeemLpAccs;

use crate::PricingAccsAg;

pub type PriceLpTokensToRedeemAccsAg = PricingAccsAg<FlatFeeRedeemLpAccs>;

type FlatFeeKeysOwned = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::AccFlags;

impl PriceLpTokensToRedeemAccs for PriceLpTokensToRedeemAccsAg {
    type KeysOwned = PricingAccsAg<FlatFeeKeysOwned>;
    type AccFlags = PricingAccsAg<FlatFeeAccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        match self {
            Self::FlatFee(p) => PricingAccsAg::FlatFee(p.suf_keys_owned()),
        }
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAccsAg::FlatFee(p.suf_is_writer()),
        }
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAccsAg::FlatFee(p.suf_is_signer()),
        }
    }
}

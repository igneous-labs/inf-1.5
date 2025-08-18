use inf1_pp_core::traits::deprecated::PriceLpTokensToRedeemAccs;
use inf1_pp_flatfee_core::instructions::pricing::lp::redeem::FlatFeeRedeemLpAccs;
use inf1_pp_flatslab_core::instructions::pricing::FlatSlabPpAccs;

use crate::PricingAg;

pub type PriceLpTokensToRedeemAccsAg = PricingAg<FlatFeeRedeemLpAccs, FlatSlabPpAccs>;

type FlatFeeKeysOwned = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::AccFlags;

type FlatSlabKeysOwned = <FlatSlabPpAccs as PriceLpTokensToRedeemAccs>::KeysOwned;
type FlatSlabAccFlags = <FlatSlabPpAccs as PriceLpTokensToRedeemAccs>::AccFlags;

impl PriceLpTokensToRedeemAccs for PriceLpTokensToRedeemAccsAg {
    type KeysOwned = PricingAg<FlatFeeKeysOwned, FlatSlabKeysOwned>;
    type AccFlags = PricingAg<FlatFeeAccFlags, FlatSlabAccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_keys_owned()),
            Self::FlatSlab(p) => PricingAg::FlatSlab(p.suf_keys_owned()),
        }
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_writer()),
            Self::FlatSlab(p) => PricingAg::FlatSlab(p.suf_is_writer()),
        }
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_signer()),
            Self::FlatSlab(p) => PricingAg::FlatSlab(p.suf_is_signer()),
        }
    }
}

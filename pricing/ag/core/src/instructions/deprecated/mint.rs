use inf1_pp_core::traits::deprecated::PriceLpTokensToMintAccs;
use inf1_pp_flatfee_core::instructions::pricing::lp::mint::FlatFeeMintLpAccs;

use crate::PricingAccsAg;

pub type PriceLpTokensToMintAccsAg = PricingAccsAg<FlatFeeMintLpAccs>;

type FlatFeeKeysOwned = <FlatFeeMintLpAccs as PriceLpTokensToMintAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeeMintLpAccs as PriceLpTokensToMintAccs>::AccFlags;

impl PriceLpTokensToMintAccs for PriceLpTokensToMintAccsAg {
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

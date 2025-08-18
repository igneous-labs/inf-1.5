use inf1_pp_core::traits::deprecated::PriceLpTokensToMintAccs;
use inf1_pp_flatfee_core::instructions::pricing::lp::mint::FlatFeeMintLpAccs;
use inf1_pp_flatslab_core::instructions::pricing::FlatSlabPpAccs;

use crate::PricingAg;

pub type PriceLpTokensToMintAccsAg = PricingAg<FlatFeeMintLpAccs, FlatSlabPpAccs>;

type FlatFeeKeysOwned = <FlatFeeMintLpAccs as PriceLpTokensToMintAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeeMintLpAccs as PriceLpTokensToMintAccs>::AccFlags;

type FlatSlabKeysOwned = <FlatSlabPpAccs as PriceLpTokensToMintAccs>::KeysOwned;
type FlatSlabAccFlags = <FlatSlabPpAccs as PriceLpTokensToMintAccs>::AccFlags;

impl PriceLpTokensToMintAccs for PriceLpTokensToMintAccsAg {
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

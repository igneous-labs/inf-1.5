use inf1_pp_core::traits::deprecated::PriceLpTokensToRedeemAccs;
use inf1_pp_flatfee_core::instructions::pricing::lp::redeem::FlatFeeRedeemLpAccs;
use inf1_pp_flatslab_core::instructions::pricing::FlatSlabPpAccs;
use inf1_pp_reserve_v2_core::instructions::pricing::ReserveV2PpAccs;

use crate::{internal_utils::map_variant, PricingAg};

pub type PriceLpTokensToRedeemAccsAg =
    PricingAg<FlatFeeRedeemLpAccs, FlatSlabPpAccs, ReserveV2PpAccs>;

type FlatFeeKeysOwned = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeeRedeemLpAccs as PriceLpTokensToRedeemAccs>::AccFlags;

type FlatSlabKeysOwned = <FlatSlabPpAccs as PriceLpTokensToRedeemAccs>::KeysOwned;
type FlatSlabAccFlags = <FlatSlabPpAccs as PriceLpTokensToRedeemAccs>::AccFlags;

type ReserveV2KeysOwned = <ReserveV2PpAccs as PriceLpTokensToRedeemAccs>::KeysOwned;
type ReserveV2AccFlags = <ReserveV2PpAccs as PriceLpTokensToRedeemAccs>::AccFlags;

impl PriceLpTokensToRedeemAccs for PriceLpTokensToRedeemAccsAg {
    type KeysOwned = PricingAg<FlatFeeKeysOwned, FlatSlabKeysOwned, ReserveV2KeysOwned>;
    type AccFlags = PricingAg<FlatFeeAccFlags, FlatSlabAccFlags, ReserveV2AccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        map_variant!(self, PriceLpTokensToRedeemAccs::suf_keys_owned)
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        map_variant!(self, PriceLpTokensToRedeemAccs::suf_is_writer)
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        map_variant!(self, PriceLpTokensToRedeemAccs::suf_is_signer)
    }
}

use inf1_ctl_core::instructions::liquidity::add::{
    AddLiquidityIxPreAccFlags, AddLiquidityIxPreKeysOwned, ADD_LIQUIDITY_IX_PRE_IS_SIGNER,
    ADD_LIQUIDITY_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::PriceLpTokensToMintAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type AddLiquidityIxAccs<I, C, P> = IxAccs<I, C, P>;

pub type AddLiquidityIxArgs<C, P> = IxArgs<C, P>;

impl<C: SolValCalcAccs, P: PriceLpTokensToMintAccs>
    AddLiquidityIxAccs<AddLiquidityIxPreKeysOwned, C, P>
{
    /// Use return value with [`super::accs_seq`] to create array
    #[inline]
    pub fn to_keys_owned(
        &self,
    ) -> AddLiquidityIxAccs<AddLiquidityIxPreKeysOwned, C::KeysOwned, P::KeysOwned> {
        IxAccs {
            ix_prefix: self.ix_prefix,
            lst_calc: self.lst_calc.suf_keys_owned(),
            pricing: self.pricing.suf_keys_owned(),
        }
    }

    /// Use return value with [`super::accs_seq`] to create array
    #[inline]
    pub fn to_is_signer(
        &self,
    ) -> AddLiquidityIxAccs<AddLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
        IxAccs {
            ix_prefix: ADD_LIQUIDITY_IX_PRE_IS_SIGNER,
            lst_calc: self.lst_calc.suf_is_signer(),
            pricing: self.pricing.suf_is_signer(),
        }
    }

    /// Use return value with [`super::accs_seq`] to create array
    #[inline]
    pub fn to_is_writer(
        &self,
    ) -> AddLiquidityIxAccs<AddLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
        IxAccs {
            ix_prefix: ADD_LIQUIDITY_IX_PRE_IS_WRITER,
            lst_calc: self.lst_calc.suf_is_writer(),
            pricing: self.pricing.suf_is_writer(),
        }
    }
}

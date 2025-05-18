//! SolValCalc interface instructions

use inf1_svc_core::traits::SolValCalcAccs;
use inf1_svc_generic::instructions::{
    IxSufAccFlags, IxSufKeysOwned, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER,
};
use solido_legacy_core::LIDO_STATE_ADDR;

use crate::keys::{POOL_PROGDATA_ID, POOL_PROG_ID, STATE_ID};

pub const IX_SUF_KEYS_OWNED: IxSufKeysOwned = IxSufKeysOwned::memset([0u8; 32])
    .const_with_pool_prog(POOL_PROG_ID)
    .const_with_pool_progdata(POOL_PROGDATA_ID)
    .const_with_pool_state(LIDO_STATE_ADDR)
    .const_with_state(STATE_ID);

pub const LST_TO_SOL_IX_SUF_KEYS: IxSufKeysOwned = IX_SUF_KEYS_OWNED;

pub const SOL_TO_LST_IX_SUF_KEYS: IxSufKeysOwned = IX_SUF_KEYS_OWNED;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LidoCalcAccs;

impl SolValCalcAccs for LidoCalcAccs {
    type KeysOwned = IxSufKeysOwned;

    type AccFlags = IxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.svcp_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.svcp_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.svcp_suf_is_signer()
    }
}

impl LidoCalcAccs {
    #[inline]
    pub const fn svcp_suf_keys_owned(&self) -> IxSufKeysOwned {
        IX_SUF_KEYS_OWNED
    }

    #[inline]
    pub const fn svcp_suf_is_writer(&self) -> IxSufAccFlags {
        IX_SUF_IS_WRITER
    }

    #[inline]
    pub const fn svcp_suf_is_signer(&self) -> IxSufAccFlags {
        IX_SUF_IS_SIGNER
    }
}

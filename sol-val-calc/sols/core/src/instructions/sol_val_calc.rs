//! SolValCalc interface instructions

use inf1_svc_core::traits::SolValCalcAccs;
use inf1_svc_generic::instructions::interface::{
    IxSufAccFlags, IxSufKeysOwned, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER,
};

use crate::keys::{CONST_KEYS_OWNED, CONST_PDAS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SolsCalcAccs {
    pub sols_pool_addr: [u8; 32],
}

/// Constructors
impl SolsCalcAccs {
    #[inline]
    pub const fn of_sols_pool_addr(sols_pool_addr: &[u8; 32]) -> &Self {
        // safety: repr(transparent) means cast is valid
        unsafe { &*sols_pool_addr.as_ptr().cast() }
    }
}

/// SolValCalcAccs
impl SolsCalcAccs {
    const BASE_KEYS_OWNED: IxSufKeysOwned = IxSufKeysOwned::memset([0u8; 32])
        .const_with_pool_prog(*CONST_KEYS_OWNED.pool_prog())
        .const_with_pool_progdata(CONST_PDAS.pool_progdata().0)
        .const_with_state(CONST_PDAS.state().0);

    #[inline]
    pub const fn svc_suf_keys_owned(&self) -> IxSufKeysOwned {
        Self::BASE_KEYS_OWNED.const_with_pool_state(self.sols_pool_addr)
    }

    #[inline]
    pub const fn svc_suf_is_writer(&self) -> IxSufAccFlags {
        IX_SUF_IS_WRITER
    }

    #[inline]
    pub const fn svc_suf_is_signer(&self) -> IxSufAccFlags {
        IX_SUF_IS_SIGNER
    }
}

impl SolValCalcAccs for SolsCalcAccs {
    type KeysOwned = IxSufKeysOwned;

    type AccFlags = IxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.svc_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.svc_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.svc_suf_is_signer()
    }
}

use inf1_svc_core::traits::SolValCalcAccs;
use inf1_svc_generic::instructions::{
    IxSufAccFlags, IxSufKeysOwned, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER,
};

macro_rules! calc_accs {
    (
        $Ty:ident,
        $progmod:ident
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $Ty {
            pub stake_pool_addr: [u8; 32],
        }

        /// Constructors
        impl $Ty {
            #[inline]
            pub const fn of_stake_pool_addr(stake_pool_addr: &[u8; 32]) -> &Self {
                // safety: repr(transparent) means cast is valid
                unsafe { &*stake_pool_addr.as_ptr().cast() }
            }
        }

        /// SolValCalcAccs
        impl $Ty {
            const BASE_KEYS_OWNED: IxSufKeysOwned = IxSufKeysOwned::memset([0u8; 32])
                .const_with_pool_prog(crate::keys::$progmod::POOL_PROG_ID)
                .const_with_pool_progdata(crate::keys::$progmod::POOL_PROGDATA_ID)
                .const_with_state(crate::keys::$progmod::STATE_ID);

            #[inline]
            pub const fn svc_suf_keys_owned(&self) -> IxSufKeysOwned {
                Self::BASE_KEYS_OWNED.const_with_pool_state(self.stake_pool_addr)
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

        impl SolValCalcAccs for $Ty {
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
    };
}

calc_accs!(SplCalcAccs, spl);
calc_accs!(SanctumSplCalcAccs, sanctum_spl);
calc_accs!(SanctumSplMultiCalcAccs, sanctum_spl_multi);

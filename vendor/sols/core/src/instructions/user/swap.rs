use generic_array_struct::generic_array_struct;

use crate::{
    instructions::{
        common::{MintPoolPair, MintPoolPairAccFlags},
        internal_utils::{csi_at, SwapIxData as GenSwapIxData},
        user::{
            BTCIxSufAccs, MintClaimCoreAccs, BTC_IX_SUF_IS_SIGNER, BTC_IX_SUF_IS_WRITER,
            TTM_IX_PRE_IS_SIGNER, TTM_IX_PRE_IS_WRITER,
        },
    },
    internal_utils::{impl_asref, impl_memset},
    keys::CONST_KEYS_OWNED,
};

// Accounts

pub const SWAP_INP_IX_PRE_IS_WRITER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(true);

pub const SWAP_INP_IX_PRE_IS_SIGNER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(false);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapSolProgAccs<T> {
    /// tokenkeg program
    pub token: T,
}

impl_memset!(SwapSolProgAccs);
impl_asref!(SwapSolProgAccs);

pub const SWAP_SOL_PROG_KEYS_OWNED: SwapSolProgAccs<[u8; 32]> =
    SwapSolProgAccs::const_from_destr(SwapSolProgAccsDestr {
        token: *CONST_KEYS_OWNED.tokenkeg_prog(),
    });

pub const SWAP_SOL_PROG_IS_SIGNER: SwapSolProgAccs<bool> = SwapSolProgAccs::memset(false);

pub const SWAP_SOL_PROG_IS_WRITER: SwapSolProgAccs<bool> = SwapSolProgAccs::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapIxAccs<P, S, X, Y> {
    /// [`MintPoolPair`]
    pub inp_pre: P,

    /// [`BTCIxSufAccs`]
    pub inp_suf: S,

    /// [`MintClaimCoreAccs`]
    pub out: X,

    /// [`SwapSolProgAccs`]
    pub progs: Y,
}

pub type SwapIxGen<T> =
    SwapIxAccs<MintPoolPair<T>, BTCIxSufAccs<T>, MintClaimCoreAccs<T>, SwapSolProgAccs<T>>;

pub const SWAP_IX_IS_WRITER: SwapIxGen<bool> = SwapIxAccs {
    inp_pre: SWAP_INP_IX_PRE_IS_WRITER,
    inp_suf: BTC_IX_SUF_IS_WRITER,
    out: TTM_IX_PRE_IS_WRITER,
    progs: SWAP_SOL_PROG_IS_WRITER,
};

pub const SWAP_IX_IS_SIGNER: SwapIxGen<bool> = SwapIxAccs {
    inp_pre: SWAP_INP_IX_PRE_IS_SIGNER,
    inp_suf: BTC_IX_SUF_IS_SIGNER,
    out: TTM_IX_PRE_IS_SIGNER,
    progs: SWAP_SOL_PROG_IS_SIGNER,
};

pub type SwapIxAccsIter<'a, T> = csi_at!(@ @ @);

impl<T> SwapIxGen<T> {
    #[inline]
    pub fn seq(&self) -> SwapIxAccsIter<'_, T> {
        self.inp_pre
            .0
            .iter()
            .chain(self.inp_suf.0.iter())
            .chain(self.out.0.iter())
            .chain(self.progs.0.iter())
    }
}

// Data

pub const SWAP_IX_DISCM: u8 = 7;

pub const SWAP_IX_DATA_LEN: usize = SwapIxData::LEN;

pub type SwapIxData = GenSwapIxData<SWAP_IX_DISCM>;

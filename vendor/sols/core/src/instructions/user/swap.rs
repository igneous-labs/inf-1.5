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
    typedefs::{TokenSolEmpty, TokenT},
    utils::InpOut,
};

// Accounts

pub const SWAP_INP_IX_PRE_IS_WRITER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(true);

pub const SWAP_INP_IX_PRE_IS_SIGNER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(false);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapSolProgAccs<T> {
    /// The SOLS program. Identity of this account is used to discriminate
    /// between the 2 swap variants
    pub this: T,

    /// tokenkeg program
    pub token: T,
}

impl_memset!(SwapSolProgAccs);
impl_asref!(SwapSolProgAccs);

pub const SWAP_SOL_PROG_KEYS_OWNED: SwapSolProgAccs<[u8; 32]> =
    SwapSolProgAccs::const_from_destr(SwapSolProgAccsDestr {
        this: *CONST_KEYS_OWNED.program_id(),
        token: *CONST_KEYS_OWNED.tokenkeg_prog(),
    });

pub const SWAP_SOL_PROG_IS_SIGNER: SwapSolProgAccs<bool> = SwapSolProgAccs::memset(false);

pub const SWAP_SOL_PROG_IS_WRITER: SwapSolProgAccs<bool> = SwapSolProgAccs::memset(false);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SwapOtherPreAccs<T> {
    /// protocol PDA
    pub protocol: T,

    /// mint of the intermediate holding token
    pub holding_mint: T,
}

impl_memset!(SwapOtherPreAccs);
impl_asref!(SwapOtherPreAccs);

pub const SWAP_OTHER_PRE_IS_SIGNER: SwapOtherPreAccs<bool> = SwapOtherPreAccs::memset(false);

pub const SWAP_OTHER_PRE_IS_WRITER: SwapOtherPreAccs<bool> = SwapOtherPreAccs::memset(false);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SwapOtherPoolAuxAccs<T> {
    /// The pool's portfolio PDA
    pub portfolio: T,

    /// The pool's ATA for the holding token
    pub holding_ata: T,
}

impl_memset!(SwapOtherPoolAuxAccs);
impl_asref!(SwapOtherPoolAuxAccs);

pub const SWAP_OTHER_POOL_AUX_ACCS_IS_SIGNER: SwapOtherPoolAuxAccs<bool> =
    SwapOtherPoolAuxAccs::memset(false);

pub const SWAP_OTHER_POOL_AUX_ACCS_IS_WRITER: SwapOtherPoolAuxAccs<bool> =
    SwapOtherPoolAuxAccs::memset(true);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SwapOtherProgAccs<T> {
    /// Holding token's SOL value calculator program acc
    pub svc: T,

    /// The holding token's token program
    pub holding_token: T,

    /// tokenkeg program
    pub token: T,
}

impl_memset!(SwapOtherProgAccs);
impl_asref!(SwapOtherProgAccs);

pub const SWAP_OTHER_PROG_ACCS_IS_SIGNER: SwapOtherProgAccs<bool> =
    SwapOtherProgAccs::memset(false);

pub const SWAP_OTHER_PROG_ACCS_IS_WRITER: SwapOtherProgAccs<bool> =
    SwapOtherProgAccs::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapOtherAccs<P, I, S, X> {
    /// [`SwapOtherPreAccs`]
    pub pre: P,

    /// [`InpOut<SwapOtherPoolAuxAccs>`]
    pub inp_out: I,

    pub svc_suf: S,

    /// [`SwapOtherProgAccs`]
    pub progs: X,
}

pub type SwapOtherGen<T, S> =
    SwapOtherAccs<SwapOtherPreAccs<T>, InpOut<SwapOtherPoolAuxAccs<T>>, S, SwapOtherProgAccs<T>>;

#[inline]
pub const fn swap_other_is_writer<S>(svc_suf: S) -> SwapOtherGen<bool, S> {
    SwapOtherGen {
        pre: SWAP_OTHER_PRE_IS_WRITER,
        inp_out: InpOut::memset(SWAP_OTHER_POOL_AUX_ACCS_IS_WRITER),
        svc_suf,
        progs: SWAP_OTHER_PROG_ACCS_IS_WRITER,
    }
}

#[inline]
pub const fn swap_other_is_signer<S>(svc_suf: S) -> SwapOtherGen<bool, S> {
    SwapOtherGen {
        pre: SWAP_OTHER_PRE_IS_SIGNER,
        inp_out: InpOut::memset(SWAP_OTHER_POOL_AUX_ACCS_IS_SIGNER),
        svc_suf,
        progs: SWAP_OTHER_PROG_ACCS_IS_SIGNER,
    }
}

impl<P, I, S, X> SwapOtherAccs<P, I, S, X> {
    #[inline]
    pub const fn is_writer(svc_suf: S) -> SwapOtherGen<bool, S> {
        swap_other_is_writer(svc_suf)
    }

    #[inline]
    pub const fn is_signer(svc_suf: S) -> SwapOtherGen<bool, S> {
        swap_other_is_signer(svc_suf)
    }
}

pub type SwapCtlAccs<T, S> = TokenT<SwapSolProgAccs<T>, SwapOtherGen<T, S>>;

impl<S> SwapCtlAccs<[u8; 32], S> {
    pub const SOL_KEYS_OWNED: Self = Self::Sol(SWAP_SOL_PROG_KEYS_OWNED);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapIxAccs<P, S, X, Y> {
    /// [`MintPoolPair`]
    pub inp_pre: P,

    /// [`BTCIxSufAccs`]
    pub inp_suf: S,

    /// [`MintClaimCoreAccs`]
    pub out: X,

    /// [`SwapCtlAccs`]
    pub ctl: Y,
}

pub type SwapIxGen<T, OtherSvcSuf> =
    SwapIxAccs<MintPoolPair<T>, BTCIxSufAccs<T>, MintClaimCoreAccs<T>, SwapCtlAccs<T, OtherSvcSuf>>;

// cant be const due to destructor of svc_suf
#[inline]
pub fn swap_ix_is_writer<T, S>(svc_suf: TokenT<T, S>) -> SwapIxGen<bool, S> {
    SwapIxAccs {
        inp_pre: SWAP_INP_IX_PRE_IS_WRITER,
        inp_suf: BTC_IX_SUF_IS_WRITER,
        out: TTM_IX_PRE_IS_WRITER,
        ctl: svc_suf.map_sep(|_| SWAP_SOL_PROG_IS_WRITER, swap_other_is_writer),
    }
}

// cant be const due to destructor of svc_suf
#[inline]
pub fn swap_ix_is_signer<T, S>(svc_suf: TokenT<T, S>) -> SwapIxGen<bool, S> {
    SwapIxAccs {
        inp_pre: SWAP_INP_IX_PRE_IS_SIGNER,
        inp_suf: BTC_IX_SUF_IS_SIGNER,
        out: TTM_IX_PRE_IS_SIGNER,
        ctl: svc_suf.map_sep(|_| SWAP_SOL_PROG_IS_SIGNER, swap_other_is_signer),
    }
}

impl<P, S, X, Y> SwapIxAccs<P, S, X, Y> {
    #[inline]
    pub fn is_writer(svc_suf: TokenSolEmpty<S>) -> SwapIxGen<bool, S> {
        swap_ix_is_writer(svc_suf)
    }

    #[inline]
    pub fn is_signer(svc_suf: TokenSolEmpty<S>) -> SwapIxGen<bool, S> {
        swap_ix_is_signer(svc_suf)
    }
}

pub type SwapIxAccsIter<'a, T> = csi_at!(@ @ @ @ @ @ @);

impl<T, S: AsRef<[T]>> SwapIxGen<T, S> {
    #[inline]
    pub fn seq(&self) -> SwapIxAccsIter<'_, T> {
        let [s0, s1, s2, s3, s4] = self.ctl.map_comb_ref(
            |s| [s.0.as_slice(), &[], &[], &[], &[]],
            |SwapOtherAccs {
                 pre,
                 inp_out,
                 svc_suf,
                 progs,
             }| {
                [
                    &pre.0,
                    &inp_out.inp().0,
                    &inp_out.out().0,
                    svc_suf.as_ref(),
                    &progs.0,
                ]
            },
        );
        self.inp_pre
            .0
            .iter()
            .chain(self.inp_suf.0.iter())
            .chain(self.out.0.iter())
            .chain(s0)
            .chain(s1)
            .chain(s2)
            .chain(s3)
            .chain(s4)
    }
}

// Data

pub const SWAP_IX_DISCM: u8 = 7;

pub const SWAP_IX_DATA_LEN: usize = SwapIxData::LEN;

pub type SwapIxData = GenSwapIxData<SWAP_IX_DISCM>;

use generic_array_struct::generic_array_struct;

use crate::{
    const_map_rebal_sol_empty,
    instructions::{
        common::{PoolHoldingAccFlags, HOLDING_PROG_IS_SIGNER, HOLDING_PROG_IS_WRITER},
        internal_utils::{caba, csba, csi_at, discm_checked},
        rebal::{RebalHoldingProgAccs, RebalPoolHolding, RebalSvcProgAcc},
        user::{
            ClaimHoldingIxSufAccs, CLAIM_HOLDING_IX_SUF_IS_SIGNER, CLAIM_HOLDING_IX_SUF_IS_WRITER,
        },
    },
    internal_utils::impl_memset,
    typedefs::TokenSolEmpty,
    utils::{InpOut, InpOutDestr},
};

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StartRebalIxPreAccs<T> {
    /// Pool rebalancer or free acc if this is a permissionless liquidation
    pub signer: T,

    /// Account to receive pool's out tokens. Can be any account if out=SOL
    pub out: T,

    /// Pool being rebalanced
    pub pool: T,

    /// Instructions sysvar
    pub instructions: T,
}

impl_memset!(StartRebalIxPreAccs);

pub type StartRebalIxPreKeys<'a> = StartRebalIxPreAccs<&'a [u8; 32]>;
pub type StartRebalIxPreKeysOwned = StartRebalIxPreAccs<[u8; 32]>;
pub type StartRebalIxPreAccFlags = StartRebalIxPreAccs<bool>;

pub const START_REBAL_IX_PRE_IS_WRITER: StartRebalIxPreAccFlags =
    StartRebalIxPreAccFlags::memset(true)
        .const_with_signer(false)
        .const_with_instructions(false);

pub const START_REBAL_IX_PRE_IS_SIGNER: StartRebalIxPreAccFlags =
    StartRebalIxPreAccFlags::memset(false).const_with_signer(true);

pub const START_REBAL_IX_HOLDINGS_INP_IS_WRITER: PoolHoldingAccFlags =
    PoolHoldingAccFlags::memset(false);

pub const START_REBAL_IX_HOLDINGS_OUT_IS_WRITER: PoolHoldingAccFlags =
    PoolHoldingAccFlags::memset(false).const_with_ata(true);

pub const START_REBAL_IX_HOLDINGS_IS_SIGNER: PoolHoldingAccFlags =
    PoolHoldingAccFlags::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StartRebalIxAccs<P, S, H, I, J, X, Y> {
    /// [`StartRebalIxPreAccs`]
    pub pre: P,

    /// [`ClaimHoldingIxSufAccs`]
    pub suf: S,

    /// [`InpOut<RebalPoolHolding>`]
    pub holdings: H,

    /// input holding's SOL value calculator account suffix,
    /// excluding mint. Omitted if SOL
    pub inp_svc_suf: I,

    /// output holding's SOL value calculator account suffix,
    /// excluding mint. Omitted if SOL
    pub out_svc_suf: J,

    /// [`RebalSvcProgAcc`]
    pub inp_svc_prog: X,

    /// [`RebalHoldingProgAccs`]
    pub out_progs: Y,
}

pub type StartRebalIxGen<T, I, J> = StartRebalIxAccs<
    StartRebalIxPreAccs<T>,
    ClaimHoldingIxSufAccs<T>,
    InpOut<RebalPoolHolding<T>>,
    I,
    J,
    RebalSvcProgAcc<T>,
    RebalHoldingProgAccs<T>,
>;

#[inline]
pub const fn start_rebal_ix_is_writer<I, J>(
    inp_svc_suf: TokenSolEmpty<I>,
    out_svc_suf: TokenSolEmpty<J>,
) -> StartRebalIxGen<bool, TokenSolEmpty<I>, TokenSolEmpty<J>> {
    let inp_svc_prog = const_map_rebal_sol_empty!(&inp_svc_suf, [false]);
    let out_progs = const_map_rebal_sol_empty!(&out_svc_suf, HOLDING_PROG_IS_WRITER);
    let holdings_inp =
        const_map_rebal_sol_empty!(&inp_svc_suf, START_REBAL_IX_HOLDINGS_INP_IS_WRITER);
    let holdings_out =
        const_map_rebal_sol_empty!(&out_svc_suf, START_REBAL_IX_HOLDINGS_OUT_IS_WRITER);
    StartRebalIxGen {
        pre: START_REBAL_IX_PRE_IS_WRITER,
        suf: CLAIM_HOLDING_IX_SUF_IS_WRITER,
        holdings: InpOut::const_from_destr(InpOutDestr {
            inp: holdings_inp,
            out: holdings_out,
        }),
        inp_svc_suf,
        out_svc_suf,
        inp_svc_prog,
        out_progs,
    }
}

#[inline]
pub const fn start_rebal_ix_is_signer<I, J>(
    inp_svc_suf: TokenSolEmpty<I>,
    out_svc_suf: TokenSolEmpty<J>,
) -> StartRebalIxGen<bool, TokenSolEmpty<I>, TokenSolEmpty<J>> {
    let inp_svc_prog = const_map_rebal_sol_empty!(&inp_svc_suf, [false]);
    let out_progs = const_map_rebal_sol_empty!(&out_svc_suf, HOLDING_PROG_IS_SIGNER);
    let holdings_inp = const_map_rebal_sol_empty!(&inp_svc_suf, START_REBAL_IX_HOLDINGS_IS_SIGNER);
    let holdings_out = const_map_rebal_sol_empty!(&out_svc_suf, START_REBAL_IX_HOLDINGS_IS_SIGNER);
    StartRebalIxGen {
        pre: START_REBAL_IX_PRE_IS_SIGNER,
        suf: CLAIM_HOLDING_IX_SUF_IS_SIGNER,
        holdings: InpOut::const_from_destr(InpOutDestr {
            inp: holdings_inp,
            out: holdings_out,
        }),
        inp_svc_suf,
        out_svc_suf,
        inp_svc_prog,
        out_progs,
    }
}

impl<P, S, H, I, J, X, Y> StartRebalIxAccs<P, S, H, I, J, X, Y> {
    #[inline]
    pub const fn is_writer(
        inp_svc_suf: TokenSolEmpty<I>,
        out_svc_suf: TokenSolEmpty<J>,
    ) -> StartRebalIxGen<bool, TokenSolEmpty<I>, TokenSolEmpty<J>> {
        start_rebal_ix_is_writer(inp_svc_suf, out_svc_suf)
    }

    #[inline]
    pub const fn is_signer(
        inp_svc_suf: TokenSolEmpty<I>,
        out_svc_suf: TokenSolEmpty<J>,
    ) -> StartRebalIxGen<bool, TokenSolEmpty<I>, TokenSolEmpty<J>> {
        start_rebal_ix_is_signer(inp_svc_suf, out_svc_suf)
    }
}

pub type StartRebalIxAccsIter<'a, T> = csi_at!(@ @ @ @ @ @ @);

pub type StartRebalIxSvcAccsIter<'a, T> = csi_at!(@);

impl<T, I, J> StartRebalIxGen<T, I, J>
where
    I: AsRef<[T]>,
    J: AsRef<[T]>,
{
    #[inline]
    pub fn seq(&self) -> StartRebalIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.suf.0.iter())
            .chain(self.holdings.inp().as_ref().iter())
            .chain(self.holdings.out().as_ref().iter())
            .chain(self.inp_svc_suf.as_ref().iter())
            .chain(self.out_svc_suf.as_ref().iter())
            .chain(self.inp_svc_prog.as_ref().iter())
            .chain(self.out_progs.as_ref().iter())
    }
}

impl<T, P, S, I, J, X, Y> StartRebalIxAccs<P, S, InpOut<RebalPoolHolding<T>>, I, J, X, Y>
where
    I: AsRef<[T]>,
    J: AsRef<[T]>,
{
    /// Returns the sequence of accounts to input to the
    /// SOL value calculator program CPI for the input token
    #[inline]
    pub fn inp_svc_accs(&self) -> StartRebalIxSvcAccsIter<'_, T> {
        match self.holdings.inp() {
            TokenSolEmpty::Sol(_) => [].iter().chain([].iter()),
            TokenSolEmpty::Other(inp) => core::slice::from_ref(inp.mint())
                .iter()
                .chain(self.inp_svc_suf.as_ref().iter()),
        }
    }

    /// Returns the sequence of accounts to input to the
    /// SOL value calculator program CPI for the output token
    #[inline]
    pub fn out_svc_accs(&self) -> StartRebalIxSvcAccsIter<'_, T> {
        match self.holdings.out() {
            TokenSolEmpty::Sol(_) => [].iter().chain([].iter()),
            TokenSolEmpty::Other(out) => core::slice::from_ref(out.mint())
                .iter()
                .chain(self.out_svc_suf.as_ref().iter()),
        }
    }
}

// StartRebalIxPreAccs just here to constrain T
impl<T, S, H, I, J, X, Y> StartRebalIxAccs<StartRebalIxPreAccs<T>, S, H, I, J, X, Y>
where
    I: AsRef<[T]>,
{
    /// Can be used for [`StartRebalIxArgs::inp_suf_len`]
    #[inline]
    pub fn try_inp_svc_suf_len(&self) -> Option<u8> {
        self.inp_svc_suf.as_ref().len().try_into().ok()
    }
}

// Data

pub const START_REBAL_IX_DISCM: u8 = 17;

pub const START_REBAL_IX_DATA_LEN: usize = 18;

// If only there was a NonMax integer type like NonZero, then this type's size will be halved
// Could handroll our own nonmax crate using NonZeroU32 xor u32::MAX but forget it, thats more CUs
pub type RebalIdx = TokenSolEmpty<u32>;

impl RebalIdx {
    #[inline]
    pub const fn into_u32(self) -> u32 {
        match self {
            Self::Other(u) => u,
            Self::Sol(_) => u32::MAX,
        }
    }

    #[inline]
    pub const fn from_u32(u: u32) -> Self {
        match u {
            u32::MAX => Self::SOL_EMPTY,
            u => Self::Other(u),
        }
    }

    #[inline]
    pub const fn into_usize(self) -> usize {
        self.into_u32() as usize
    }
}

impl From<RebalIdx> for u32 {
    #[inline]
    fn from(v: RebalIdx) -> Self {
        v.into_u32()
    }
}

impl From<u32> for RebalIdx {
    #[inline]
    fn from(v: u32) -> Self {
        Self::from_u32(v)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RebalArgs {
    pub idxs: InpOut<RebalIdx>,
    pub amt: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StartRebalIxArgs {
    /// 0 if SOL
    pub inp_suf_len: u8,
    pub args: RebalArgs,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StartRebalIxData([u8; START_REBAL_IX_DATA_LEN]);

impl StartRebalIxData {
    pub const LEN: usize = START_REBAL_IX_DATA_LEN;

    #[inline]
    pub const fn new(
        StartRebalIxArgs {
            inp_suf_len,
            args: RebalArgs { idxs, amt },
        }: &StartRebalIxArgs,
    ) -> Self {
        const A: usize = START_REBAL_IX_DATA_LEN;

        let mut res = [0; A];

        res = caba::<A, 0, 1>(res, &[START_REBAL_IX_DISCM]);
        res = caba::<A, 1, 1>(res, &[*inp_suf_len]);
        res = caba::<A, 2, 4>(res, &idxs.inp().into_u32().to_le_bytes());
        res = caba::<A, 6, 4>(res, &idxs.out().into_u32().to_le_bytes());
        res = caba::<A, 10, 8>(res, &amt.to_le_bytes());

        Self(res)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; START_REBAL_IX_DATA_LEN] {
        &self.0
    }

    #[inline]
    pub const fn parse_no_discm(data: &[u8; START_REBAL_IX_DATA_LEN - 1]) -> StartRebalIxArgs {
        let ([inp_suf_len], data) = csba::<17, 1, 16>(data);
        let (inp, data) = csba::<16, 4, 12>(data);
        let (out, data) = csba::<12, 4, 8>(data);
        let (amt, _) = csba::<8, 8, 0>(data);
        StartRebalIxArgs {
            inp_suf_len: *inp_suf_len,
            args: RebalArgs {
                idxs: InpOut::const_from_destr(InpOutDestr {
                    inp: RebalIdx::from_u32(u32::from_le_bytes(*inp)),
                    out: RebalIdx::from_u32(u32::from_le_bytes(*out)),
                }),
                amt: u64::from_le_bytes(*amt),
            },
        }
    }

    /// Returns `None` if discm does not match
    #[inline]
    pub const fn parse(data: &[u8; START_REBAL_IX_DATA_LEN]) -> Option<StartRebalIxArgs> {
        match discm_checked(START_REBAL_IX_DISCM, data) {
            None => None,
            Some(d) => Some(Self::parse_no_discm(d)),
        }
    }
}

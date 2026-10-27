use crate::{
    instructions::user::SwapArgs,
    typedefs::{Nanos, NanosOutOfRangeErr},
};

// This does not seem to produce different bytecode
// on-chain compared to .copy_from_slice(), but it allows us to retain `const`
/// caba = `const_assign_byte_array`
#[inline]
pub(crate) const fn caba<const A: usize, const START: usize, const LEN: usize>(
    mut arr: [u8; A],
    val: &[u8; LEN],
) -> [u8; A] {
    const {
        assert!(START + LEN <= A);
    }

    let mut i = 0;
    while i < LEN {
        arr[START + i] = val[i];
        i += 1;
    }
    arr
}

/// csba = `const_split_byte_array`
#[inline]
pub(crate) const fn csba<const M: usize, const N: usize, const X: usize>(
    data: &[u8; M],
) -> (&[u8; N], &[u8; X]) {
    const {
        assert!(N <= M);
        assert!(X == M - N)
    }

    // Safety: bounds checked above
    let (a, b) = unsafe { data.split_at_unchecked(N) };

    // SAFETY: data is guaranteed to be of length M
    // and we are splitting it into two slices of length N and X (i.e M-N)
    (unsafe { &*a.as_ptr().cast::<[u8; N]>() }, unsafe {
        &*b.as_ptr().cast::<[u8; X]>()
    })
}

/// Returns `None` if discm does not match first byte, Some(rest of data) otherwise
#[inline]
pub(crate) const fn discm_checked<const M: usize, const D: usize>(
    expected_discm: u8,
    data: &[u8; M],
) -> Option<&[u8; D]> {
    let ([discm], data) = csba::<M, 1, D>(data);
    if *discm != expected_discm {
        return None;
    }
    Some(data)
}

pub const U64_OPT_IX_DATA_BUF_LEN: usize = 10;

/// Instruction data where the single u64 le arg is optional
/// and omitted if None.
///
/// ## Layout
///
/// Layout of this struct itself, not to be confused with the actual ix data bytes that
/// is sent in the transaction. See [`Self::as_buf`] for that.
///
/// - first 9 bytes is possible actual instruction data (discm + u64 arg if present)
/// - last byte is an option discriminant that is not included in the ix data.
///     - if 0, bytes[1..9] should be ignored
///     - if anything else, bytes[1..9] should be treated as the u64 arg
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct U64OptIxData<const DISCM: u8>([u8; U64_OPT_IX_DATA_BUF_LEN]);

impl<const DISCM: u8> U64OptIxData<DISCM> {
    pub const LEN: usize = U64_OPT_IX_DATA_BUF_LEN;

    pub const NONE: Self = {
        let mut res = [0; U64_OPT_IX_DATA_BUF_LEN];
        res[0] = DISCM;
        Self(res)
    };

    #[inline]
    pub const fn some(arg: u64) -> Self {
        const A: usize = U64_OPT_IX_DATA_BUF_LEN;

        let mut res = [0; A];

        res = caba::<A, 0, 1>(res, &[DISCM]);
        res = caba::<A, 1, 8>(res, &arg.to_le_bytes());
        res = caba::<A, 9, 1>(res, &[1]);

        Self(res)
    }

    #[inline]
    pub const fn none() -> Self {
        Self::NONE
    }

    #[inline]
    pub const fn from_opt(opt: Option<u64>) -> Self {
        match opt {
            Some(arg) => Self::some(arg),
            None => Self::none(),
        }
    }

    /// Returns the actual instruction data byte buffer that should be
    /// sent in the transaction
    #[inline]
    pub const fn as_buf(&self) -> &[u8] {
        const A: usize = U64_OPT_IX_DATA_BUF_LEN;

        let (pre, [opt_discm]) = csba::<A, 9, 1>(&self.0);
        match *opt_discm {
            0 => {
                let (discm_only, _) = csba::<9, 1, 8>(pre);
                discm_only
            }
            _ => pre,
        }
    }

    /// Returns `Err` if data is neither empty nor 8-bytes long
    #[inline]
    pub const fn parse_no_discm(data: &[u8]) -> Result<Option<u64>, ()> {
        match data.len() {
            0 => Ok(None),
            8 => {
                // safety: len was just verified
                let casted: &[u8; 8] = unsafe { &*data.as_ptr().cast() };
                Ok(Some(u64::from_le_bytes(*casted)))
            }
            _ => Err(()),
        }
    }

    /// ## Returns
    /// - `None` if discm does not match
    /// - `Some(Err)` if [`Self::parse_no_discm`] fails
    #[inline]
    pub const fn parse(data: &[u8]) -> Option<Result<Option<u64>, ()>> {
        let (discm, data) = match data.split_first() {
            None => return None,
            Some(x) => x,
        };
        if *discm != DISCM {
            return None;
        }
        Some(Self::parse_no_discm(data))
    }
}

pub const SWAP_IX_DATA_BUF_LEN: usize = 18;

/// Instruction data with
/// - required `limit` slippage control u64 le
/// - optional `amt` u64 le, omitted if None, similar to [`U64OptIxData`]
///
/// ## Layout
///
/// Layout of this struct itself, not to be confused with the actual ix data bytes that
/// is sent in the transaction. See [`Self::as_buf`] for that.
///
/// - first 17 bytes is possible actual instruction data (discm + `min_aount` arg + `amt` if present)
/// - last byte is an option discriminant that is not included in the ix data.
///     - if 0, bytes[9..17] should be ignored
///     - if anything else, bytes[9..17] should be treated as the u64 arg
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SwapIxData<const DISCM: u8>([u8; SWAP_IX_DATA_BUF_LEN]);

impl<const DISCM: u8> SwapIxData<DISCM> {
    pub const LEN: usize = SWAP_IX_DATA_BUF_LEN;

    #[inline]
    pub const fn new(SwapArgs { limit, amt }: &SwapArgs) -> Self {
        const A: usize = SWAP_IX_DATA_BUF_LEN;

        let mut res = [0; A];

        res = caba::<A, 0, 1>(res, &[DISCM]);
        res = caba::<A, 1, 8>(res, &limit.to_le_bytes());

        res = match amt {
            Some(amt_val) => {
                res = caba::<A, 9, 8>(res, &amt_val.to_le_bytes());
                caba::<A, 17, 1>(res, &[1])
            }
            None => caba::<A, 17, 1>(res, &[0]),
        };

        Self(res)
    }

    /// Returns the actual instruction data byte buffer that should be
    /// sent in the transaction
    #[inline]
    pub const fn as_buf(&self) -> &[u8] {
        const A: usize = SWAP_IX_DATA_BUF_LEN;

        let (pre, [opt_discm]) = csba::<A, 17, 1>(&self.0);
        match *opt_discm {
            0 => {
                let (none, _) = csba::<17, 9, 8>(pre);
                none
            }
            _ => pre,
        }
    }

    /// Returns `Err` if data is not of the right length
    #[inline]
    pub const fn parse_no_discm(data: &[u8]) -> Result<SwapArgs, ()> {
        match data.len() {
            8 => {
                // safety: len was just verified
                let casted: &[u8; 8] = unsafe { &*data.as_ptr().cast() };
                Ok(SwapArgs {
                    limit: u64::from_le_bytes(*casted),
                    amt: None,
                })
            }
            16 => {
                // safety: len was just verified
                let casted: &[u8; 16] = unsafe { &*data.as_ptr().cast() };
                let (limit, amt) = csba::<16, 8, 8>(casted);
                Ok(SwapArgs {
                    limit: u64::from_le_bytes(*limit),
                    amt: Some(u64::from_le_bytes(*amt)),
                })
            }
            _ => Err(()),
        }
    }

    /// ## Returns
    /// - `None` if discm does not match
    /// - `Some(Err)` if [`Self::parse_no_discm`] fails
    #[inline]
    pub const fn parse(data: &[u8]) -> Option<Result<SwapArgs, ()>> {
        let (discm, data) = match data.split_first() {
            None => return None,
            Some(x) => x,
        };
        if *discm != DISCM {
            return None;
        }
        Some(Self::parse_no_discm(data))
    }
}

pub const DISCM_ONLY_IX_DATA_LEN: usize = 1;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiscmOnlyIxData<const DISCM: u8>([u8; DISCM_ONLY_IX_DATA_LEN]);

impl<const DISCM: u8> DiscmOnlyIxData<DISCM> {
    pub const LEN: usize = DISCM_ONLY_IX_DATA_LEN;

    pub const SELF: Self = Self::new();

    #[inline]
    pub const fn new() -> Self {
        Self([DISCM])
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; DISCM_ONLY_IX_DATA_LEN] {
        &self.0
    }

    /// Returns `None` if discm does not match
    #[inline]
    pub const fn parse(data: &[u8; DISCM_ONLY_IX_DATA_LEN]) -> Option<()> {
        match discm_checked(DISCM, data) {
            None => None,
            Some([]) => Some(()),
        }
    }
}

impl<const DISCM: u8> Default for DiscmOnlyIxData<DISCM> {
    #[inline]
    fn default() -> Self {
        Self::SELF
    }
}

pub const NANO_PAIR_IX_DATA_LEN: usize = 9;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NanoPairIxData<const DISCM: u8>([u8; NANO_PAIR_IX_DATA_LEN]);

impl<const DISCM: u8> NanoPairIxData<DISCM> {
    pub const LEN: usize = NANO_PAIR_IX_DATA_LEN;

    #[inline]
    pub const fn new_raw([a, b]: &[u32; 2]) -> Self {
        const A: usize = NANO_PAIR_IX_DATA_LEN;

        let mut res = [0; A];

        res = caba::<A, 0, 1>(res, &[DISCM]);
        res = caba::<A, 1, 4>(res, &a.to_le_bytes());
        res = caba::<A, 5, 4>(res, &b.to_le_bytes());

        Self(res)
    }

    #[inline]
    pub const fn new_checked([a, b]: &[Nanos; 2]) -> Self {
        Self::new_raw(&[a.get(), b.get()])
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; NANO_PAIR_IX_DATA_LEN] {
        &self.0
    }

    #[inline]
    pub const fn parse_no_discm_raw(data: &[u8; NANO_PAIR_IX_DATA_LEN - 1]) -> [u32; 2] {
        let (a, b) = csba::<8, 4, 4>(data);
        [u32::from_le_bytes(*a), u32::from_le_bytes(*b)]
    }

    #[inline]
    pub const fn parse_no_discm_checked(
        data: &[u8; NANO_PAIR_IX_DATA_LEN - 1],
    ) -> Result<[Nanos; 2], NanosOutOfRangeErr> {
        let [a, b] = Self::parse_no_discm_raw(data);
        let a = match Nanos::new(a) {
            Err(e) => return Err(e),
            Ok(x) => x,
        };
        let b = match Nanos::new(b) {
            Err(e) => return Err(e),
            Ok(x) => x,
        };
        Ok([a, b])
    }

    /// Returns `None` if discm does not match
    #[inline]
    pub const fn parse_raw(data: &[u8; NANO_PAIR_IX_DATA_LEN]) -> Option<[u32; 2]> {
        match discm_checked(DISCM, data) {
            None => None,
            Some(a) => Some(Self::parse_no_discm_raw(a)),
        }
    }

    /// ## Returns
    /// - `None` if discm does not match
    /// - `Some(Err)` if either nano out of range
    #[inline]
    pub const fn parse_checked(
        data: &[u8; NANO_PAIR_IX_DATA_LEN],
    ) -> Option<Result<[Nanos; 2], NanosOutOfRangeErr>> {
        match discm_checked(DISCM, data) {
            None => None,
            Some(a) => Some(Self::parse_no_discm_checked(a)),
        }
    }
}

pub const NANO_IX_DATA_LEN: usize = 5;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NanoIxData<const DISCM: u8>([u8; NANO_IX_DATA_LEN]);

impl<const DISCM: u8> NanoIxData<DISCM> {
    pub const LEN: usize = NANO_IX_DATA_LEN;

    #[inline]
    pub const fn new_raw(n: u32) -> Self {
        const A: usize = NANO_IX_DATA_LEN;

        let mut res = [0; A];

        res = caba::<A, 0, 1>(res, &[DISCM]);
        res = caba::<A, 1, 4>(res, &n.to_le_bytes());

        Self(res)
    }

    #[inline]
    pub const fn new(n: Nanos) -> Self {
        Self::new_raw(n.get())
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; NANO_IX_DATA_LEN] {
        &self.0
    }

    #[inline]
    pub const fn parse_no_discm_raw(data: &[u8; NANO_IX_DATA_LEN - 1]) -> u32 {
        u32::from_le_bytes(*data)
    }

    #[inline]
    pub const fn parse_no_discm(
        data: &[u8; NANO_IX_DATA_LEN - 1],
    ) -> Result<Nanos, NanosOutOfRangeErr> {
        let n = match Nanos::new(Self::parse_no_discm_raw(data)) {
            Err(e) => return Err(e),
            Ok(n) => n,
        };
        Ok(n)
    }

    /// Returns `None` if discm does not match
    #[inline]
    pub const fn parse_raw(data: &[u8; NANO_IX_DATA_LEN]) -> Option<u32> {
        match discm_checked(DISCM, data) {
            None => None,
            Some(a) => Some(Self::parse_no_discm_raw(a)),
        }
    }

    /// ## Returns
    /// - `None` if discm does not match
    /// - `Some(Err)` if either nano out of range
    #[inline]
    pub const fn parse(data: &[u8; NANO_IX_DATA_LEN]) -> Option<Result<Nanos, NanosOutOfRangeErr>> {
        match discm_checked(DISCM, data) {
            None => None,
            Some(a) => Some(Self::parse_no_discm(a)),
        }
    }
}

/// **C**hain **S**lice **I**ter for '**a** **T** generic
///
/// We want to use `Chain<slice...>` over stuff like `Flatten<array...>` because
/// the former impls TrustedLen while the latter does not. This also somewhat
/// standardizes iterator types across all instructions
///
/// Only way to make it work with decl macros is to
/// tt-munch one token each time. This is why we have `csi_at!(@ @)`
/// instead of `csi_at!(2)`
macro_rules! csi_at {
    // Recursive-case: add a Chain
    (@ $($tail:tt)*) => {
        core::iter::Chain<csi_at!($($tail)*), core::slice::Iter<'a, T>>
    };

    // Base-case: output single slice::Iter
    () => {
        core::slice::Iter<'a, T>
    };
}
pub(crate) use csi_at;

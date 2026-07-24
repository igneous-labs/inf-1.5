use generic_array_struct::generic_array_struct;

use crate::{
    errs::ReserveV2ProgramErr,
    instructions::csi_at,
    internal_utils::{caba, const_map, csba},
    typedefs::{FeeEntryNanos, FeeNanos, ThresholdNanos, ThresholdNanosOutOfRangeErr},
};

use super::common::AdminIxPreAccs;

// Accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetFeeEntryIxSufAccs<T> {
    /// accepted SPL token mint to set
    pub mint: T,

    /// funds account growth if needed
    pub payer: T,
}

impl<T: Copy> SetFeeEntryIxSufAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_FEE_ENTRY_IX_SUF_ACCS_LEN])
    }
}

impl<T> SetFeeEntryIxSufAccs<T> {
    #[inline]
    pub const fn new(arr: [T; SET_FEE_ENTRY_IX_SUF_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

pub type SetFeeEntryIxSufKeys<'a> = SetFeeEntryIxSufAccs<&'a [u8; 32]>;
pub type SetFeeEntryIxSufKeysOwned = SetFeeEntryIxSufAccs<[u8; 32]>;
pub type SetFeeEntryIxSufAccFlags = SetFeeEntryIxSufAccs<bool>;

pub const SET_FEE_ENTRY_IX_SUF_IS_WRITER: SetFeeEntryIxSufAccFlags =
    SetFeeEntryIxSufAccFlags::const_from_destr(SetFeeEntryIxSufAccsDestr {
        mint: false,
        payer: true,
    });

pub const SET_FEE_ENTRY_IX_SUF_IS_SIGNER: SetFeeEntryIxSufAccFlags =
    SetFeeEntryIxSufAccFlags::const_from_destr(SetFeeEntryIxSufAccsDestr {
        mint: false,
        payer: true,
    });

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetFeeEntryIxAccs<P, Q, T> {
    /// [`AdminIxPreAccs`]
    pub pre: P,

    /// [`SetFeeEntryIxSufAccs`]
    pub suf: Q,

    /// system program
    pub sys_prog: T,
}

pub type SetFeeEntryIxAccsGen<T> = SetFeeEntryIxAccs<AdminIxPreAccs<T>, SetFeeEntryIxSufAccs<T>, T>;

pub const SET_FEE_ENTRY_IX_IS_WRITER: SetFeeEntryIxAccsGen<bool> = SetFeeEntryIxAccs {
    pre: super::common::ADMIN_IX_PRE_IS_WRITER,
    suf: SET_FEE_ENTRY_IX_SUF_IS_WRITER,
    sys_prog: false,
};

pub const SET_FEE_ENTRY_IX_IS_SIGNER: SetFeeEntryIxAccsGen<bool> = SetFeeEntryIxAccs {
    pre: super::common::ADMIN_IX_PRE_IS_SIGNER,
    suf: SET_FEE_ENTRY_IX_SUF_IS_SIGNER,
    sys_prog: false,
};

pub type SetFeeEntryAccsIter<'a, T> = csi_at!(@ @);

impl<T> SetFeeEntryIxAccsGen<T> {
    #[inline]
    pub fn seq(&self) -> SetFeeEntryAccsIter<'_, T> {
        let Self { pre, suf, sys_prog } = self;
        pre.0
            .iter()
            .chain(suf.0.iter())
            .chain(core::slice::from_ref(sys_prog).iter())
    }
}

// Data

pub const SET_FEE_ENTRY_IX_DISCM: u8 = 253;

pub const SET_FEE_ENTRY_IX_DATA_LEN: usize = 21;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetFeeEntryIxData([u8; SET_FEE_ENTRY_IX_DATA_LEN]);

impl SetFeeEntryIxData {
    /// NB: this fn currently only used for testing program rejection of invalid values.
    /// Use [`Self::new`] instead
    ///
    /// # Safety
    /// - `threshold_nanos` must be a valid [`ThresholdNanos`]
    /// - Each field of `fees` must be a valid [`FeeNanos`]
    #[inline]
    pub const unsafe fn new_unchecked(
        threshold_nanos: u32,
        FeeEntryNanos([f0, f1, f2, f3]): FeeEntryNanos<u32>,
    ) -> Self {
        const A: usize = SET_FEE_ENTRY_IX_DATA_LEN;

        let d = [0u8; A];
        let d = caba::<A, 0, 1>(d, &[SET_FEE_ENTRY_IX_DISCM]);
        let d = caba::<A, 1, 4>(d, &threshold_nanos.to_le_bytes());
        let d = caba::<A, 5, 4>(d, &f0.to_le_bytes());
        let d = caba::<A, 9, 4>(d, &f1.to_le_bytes());
        let d = caba::<A, 13, 4>(d, &f2.to_le_bytes());
        let d = caba::<A, 17, 4>(d, &f3.to_le_bytes());
        Self(d)
    }

    #[inline]
    pub const fn new(threshold_nanos: ThresholdNanos, fees: FeeEntryNanos<FeeNanos>) -> Self {
        const fn fee_nanos_get(f: &FeeNanos) -> u32 {
            f.get()
        }
        // safety: guaranteed by types
        unsafe {
            Self::new_unchecked(
                threshold_nanos.get(),
                FeeEntryNanos(const_map!(0, fees.0, fee_nanos_get)),
            )
        }
    }

    /// Returns `None` if discm does not match
    #[inline]
    pub const fn parse(
        data: &[u8; SET_FEE_ENTRY_IX_DATA_LEN],
    ) -> Option<Result<(ThresholdNanos, FeeEntryNanos<FeeNanos>), ReserveV2ProgramErr>> {
        let ([discm], data) = csba::<21, 1, 20>(data);
        if *discm == SET_FEE_ENTRY_IX_DISCM {
            Some(Self::parse_no_discm(data))
        } else {
            None
        }
    }

    #[inline]
    pub const fn parse_no_discm(
        data: &[u8; 20],
    ) -> Result<(ThresholdNanos, FeeEntryNanos<FeeNanos>), ReserveV2ProgramErr> {
        let (threshold_bytes, data) = csba::<20, 4, 16>(data);
        let (f0, data) = csba::<16, 4, 12>(data);
        let (f1, data) = csba::<12, 4, 8>(data);
        let (f2, data) = csba::<8, 4, 4>(data);
        let (f3, _) = csba::<4, 4, 0>(data);

        let threshold = u32::from_le_bytes(*threshold_bytes);
        let threshold_nanos = match ThresholdNanos::new(threshold) {
            Ok(t) => t,
            Err(_) => {
                return Err(ReserveV2ProgramErr::ThresholdNanosOutOfRange(
                    ThresholdNanosOutOfRangeErr { actual: threshold },
                ))
            }
        };

        let fen = FeeEntryNanos([f0, f1, f2, f3]);
        const fn u32_from_le(b: &[u8; 4]) -> u32 {
            u32::from_le_bytes(*b)
        }
        let fen = FeeEntryNanos(const_map!(0, fen.0, u32_from_le));

        const fn fee_nanos_checked(f: &u32) -> Result<FeeNanos, ReserveV2ProgramErr> {
            match FeeNanos::new(*f) {
                Ok(n) => Ok(n),
                Err(e) => Err(ReserveV2ProgramErr::FeeNanosOutOfRange(e)),
            }
        }
        let res = FeeEntryNanos(const_map!(Ok(FeeNanos::ZERO), fen.0, fee_nanos_checked));
        let fee_nanos = match res.0 {
            [Err(e), ..]
            | [Ok(_), Err(e), ..]
            | [Ok(_), Ok(_), Err(e), ..]
            | [Ok(_), Ok(_), Ok(_), Err(e)] => return Err(e),
            [Ok(a), Ok(b), Ok(c), Ok(d)] => FeeEntryNanos([a, b, c, d]),
        };

        match fee_nanos.validate() {
            Ok(_) => Ok((threshold_nanos, fee_nanos)),
            Err(e) => Err(e),
        }
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; SET_FEE_ENTRY_IX_DATA_LEN] {
        &self.0
    }
}

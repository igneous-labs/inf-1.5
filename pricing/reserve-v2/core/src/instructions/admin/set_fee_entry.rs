use generic_array_struct::generic_array_struct;

use crate::{
    errs::ReserveV2ProgramErr,
    instructions::csi_at,
    internal_utils::{caba, const_map, csba},
    typedefs::{
        FeeEntryNanos, FeeEntryNanosDestr, FeeNanos, ThresholdNanos, ThresholdNanosOutOfRangeErr,
    },
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
    pub const unsafe fn new_unchecked(threshold_nanos: u32, fees: FeeEntryNanos<u32>) -> Self {
        const A: usize = SET_FEE_ENTRY_IX_DATA_LEN;

        let FeeEntryNanosDestr {
            base_fee,
            threshold_fee,
            max_fee,
            output_fee,
        } = fees.const_into_destr();
        let d = [0u8; A];
        let d = caba::<A, 0, 1>(d, &[SET_FEE_ENTRY_IX_DISCM]);
        let d = caba::<A, 1, 4>(d, &threshold_nanos.to_le_bytes());
        let d = caba::<A, 5, 4>(d, &base_fee.to_le_bytes());
        let d = caba::<A, 9, 4>(d, &threshold_fee.to_le_bytes());
        let d = caba::<A, 13, 4>(d, &max_fee.to_le_bytes());
        let d = caba::<A, 17, 4>(d, &output_fee.to_le_bytes());
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

    #[inline]
    pub const fn parse_no_discm(
        data: &[u8; 20],
    ) -> Result<(ThresholdNanos, FeeEntryNanos<FeeNanos>), ReserveV2ProgramErr> {
        let (threshold_bytes, data) = csba::<20, 4, 16>(data);
        let (base_fee_bytes, data) = csba::<16, 4, 12>(data);
        let (threshold_fee_bytes, data) = csba::<12, 4, 8>(data);
        let (max_fee_bytes, data) = csba::<8, 4, 4>(data);
        let (output_fee_bytes, _) = csba::<4, 4, 0>(data);

        const fn u32_from_le(b: &[u8; 4]) -> u32 {
            u32::from_le_bytes(*b)
        }
        let [threshold, base_fee, threshold_fee, max_fee, output_fee] = const_map!(
            0,
            [
                threshold_bytes,
                base_fee_bytes,
                threshold_fee_bytes,
                max_fee_bytes,
                output_fee_bytes
            ],
            u32_from_le
        );

        let threshold_nanos = match ThresholdNanos::new(threshold) {
            Ok(t) => t,
            Err(_) => {
                return Err(ReserveV2ProgramErr::ThresholdNanosOutOfRange(
                    ThresholdNanosOutOfRangeErr { actual: threshold },
                ))
            }
        };

        const fn fee_nanos_checked(f: &u32) -> Result<FeeNanos, ReserveV2ProgramErr> {
            match FeeNanos::new(*f) {
                Ok(n) => Ok(n),
                Err(e) => Err(ReserveV2ProgramErr::FeeNanosOutOfRange(e)),
            }
        }
        // need explicit len for match below for some reason
        let res: [_; 4] = const_map!(
            Ok(FeeNanos::ZERO),
            [base_fee, threshold_fee, max_fee, output_fee],
            fee_nanos_checked
        );
        let [base_fee, threshold_fee, max_fee, output_fee] = match res {
            [Err(e), ..]
            | [Ok(_), Err(e), ..]
            | [Ok(_), Ok(_), Err(e), ..]
            | [Ok(_), Ok(_), Ok(_), Err(e)] => return Err(e),
            [Ok(a), Ok(b), Ok(c), Ok(d)] => [a, b, c, d],
        };
        let fee_nanos = FeeEntryNanos::const_from_destr(FeeEntryNanosDestr {
            base_fee,
            threshold_fee,
            max_fee,
            output_fee,
        });

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

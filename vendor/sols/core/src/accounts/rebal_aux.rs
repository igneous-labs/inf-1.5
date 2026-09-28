use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RebalAux<M, I, B, P> {
    pub min: M,
    pub inp_idx: I,
    pub is_min_raw_bal: B,
    pub padding: P,
}

pub const REBAL_AUX_PADDING_LEN: usize = 3;

/// Type to be used in onchain program.
///
/// Use primitive val (e.g. u32 instead of Nanos) to allow for lazy verification of
/// indiv fields as needed after unverified pointer casting.
pub type RebalAuxData = RebalAux<u64, u32, u8, [u8; REBAL_AUX_PADDING_LEN]>;
impl_cast_from_acc_data!(RebalAuxData);
impl_cast_to_acc_data!(RebalAuxData);

const _ASSERT_REBAL_AUX_DATA_NO_PADDING: () = assert!(
    size_of::<RebalAuxData>()
        == size_of::<u64>()
            + size_of::<u32>()
            + size_of::<u8>()
            + size_of::<[u8; REBAL_AUX_PADDING_LEN]>()
);
const _ASSERT_REBAL_AUX_DATA_AL: () = assert!(align_of::<RebalAuxData>() == 8);

/// Packed version of [`RebalAuxData`] (align == 1)
pub type RebalAuxPked = RebalAux<[u8; 8], [u8; 4], u8, [u8; REBAL_AUX_PADDING_LEN]>;
impl_cast_from_acc_data!(RebalAuxPked, packed);
impl_cast_to_acc_data!(RebalAuxPked, packed);
const _ASSERT_POOL_V1_PKED_SZ: () = assert!(size_of::<RebalAuxPked>() == size_of::<RebalAuxData>());
const _ASSERT_POOL_V1_PKED_AL: () = assert!(align_of::<RebalAuxPked>() == 1);

impl RebalAuxData {
    #[inline]
    pub const fn from_pked(
        RebalAux {
            min,
            inp_idx,
            padding,
            is_min_raw_bal,
        }: RebalAuxPked,
    ) -> Self {
        Self {
            min: u64::from_le_bytes(min),
            inp_idx: u32::from_le_bytes(inp_idx),
            padding,
            is_min_raw_bal,
        }
    }

    #[inline]
    pub const fn into_pked(self) -> RebalAuxPked {
        let Self {
            min,
            inp_idx,
            padding,
            is_min_raw_bal,
        } = self;
        RebalAux {
            min: min.to_le_bytes(),
            inp_idx: inp_idx.to_le_bytes(),
            padding,
            is_min_raw_bal,
        }
    }
}

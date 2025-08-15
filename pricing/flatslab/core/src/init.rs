use crate::{keys::LP_MINT_ID, typedefs::SlabEntryPacked};

/// 10 bps
pub const INITIAL_LP_INP_FEE_NANOS: u32 = 1_000_000;
pub const INITIAL_LP_OUT_FEE_NANOS: u32 = 0;

impl SlabEntryPacked {
    pub const INITIAL_LP: Self = Self {
        mint: LP_MINT_ID,
        inp_fee_nanos: INITIAL_LP_INP_FEE_NANOS.to_le_bytes(),
        out_fee_nanos: INITIAL_LP_OUT_FEE_NANOS.to_le_bytes(),
    };
}

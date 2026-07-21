use crate::{
    keys::{LP_MINT, WSOL_MINT},
    typedefs::{FeeEntryNanos, FeeEntryNanosDestr, FeeEntryPacked, NANOS_DENOM},
};

pub const INITIAL_THRESHOLD_NANOS: u32 = NANOS_DENOM / 3;
pub const ZERO_FEE_NANOS: u32 = 0;

impl FeeEntryPacked {
    pub const INITIAL_LP: Self = Self {
        mint: LP_MINT,
        nanos: FeeEntryNanos::const_from_destr(FeeEntryNanosDestr {
            base_fee: ZERO_FEE_NANOS.to_le_bytes(),
            threshold: INITIAL_THRESHOLD_NANOS.to_le_bytes(),
            threshold_fee: ZERO_FEE_NANOS.to_le_bytes(),
            max_fee: ZERO_FEE_NANOS.to_le_bytes(),
            output_fee: ZERO_FEE_NANOS.to_le_bytes(),
        }),
    };

    pub const INITIAL_WSOL: Self = Self {
        mint: WSOL_MINT,
        nanos: FeeEntryNanos::const_from_destr(FeeEntryNanosDestr {
            base_fee: ZERO_FEE_NANOS.to_le_bytes(),
            threshold: INITIAL_THRESHOLD_NANOS.to_le_bytes(),
            threshold_fee: ZERO_FEE_NANOS.to_le_bytes(),
            max_fee: ZERO_FEE_NANOS.to_le_bytes(),
            output_fee: ZERO_FEE_NANOS.to_le_bytes(),
        }),
    };
}

const fn bytes_lt(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut i = 0;
    while i < 32 {
        if a[i] != b[i] {
            return a[i] < b[i];
        }
        i += 1;
    }
    false
}

pub const INITIAL_ENTRIES: [FeeEntryPacked; 2] = if bytes_lt(
    FeeEntryPacked::INITIAL_LP.mint(),
    FeeEntryPacked::INITIAL_WSOL.mint(),
) {
    [FeeEntryPacked::INITIAL_LP, FeeEntryPacked::INITIAL_WSOL]
} else {
    [FeeEntryPacked::INITIAL_WSOL, FeeEntryPacked::INITIAL_LP]
};

const _ASSERT_INITIAL_LP_VALIDATE: () = match FeeEntryPacked::INITIAL_LP.validate() {
    Ok(()) => {}
    Err(_) => panic!("invalid initial LP fee entry"),
};

const _ASSERT_INITIAL_WSOL_VALIDATE: () = match FeeEntryPacked::INITIAL_WSOL.validate() {
    Ok(()) => {}
    Err(_) => panic!("invalid initial wSOL fee entry"),
};

const _ASSERT_INITIAL_ENTRIES_SORTED: () = assert!(bytes_lt(
    INITIAL_ENTRIES[0].mint(),
    INITIAL_ENTRIES[1].mint()
));

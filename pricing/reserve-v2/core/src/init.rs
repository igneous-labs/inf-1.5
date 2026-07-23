use crate::{
    keys::{LP_MINT, WSOL_MINT},
    typedefs::{FeeEntry, FeeEntryNanos, FeeEntryNanosDestr, NANOS_DENOM},
};

pub const INITIAL_THRESHOLD_NANOS: u32 = NANOS_DENOM / 3;
pub const ZERO_FEE_NANOS: u32 = 0;

impl FeeEntry {
    pub const INITIAL_LP: Self = Self {
        mint: LP_MINT,
        nanos: FeeEntryNanos::const_from_destr(FeeEntryNanosDestr {
            base_fee: ZERO_FEE_NANOS,
            threshold: INITIAL_THRESHOLD_NANOS,
            threshold_fee: ZERO_FEE_NANOS,
            max_fee: ZERO_FEE_NANOS,
            output_fee: ZERO_FEE_NANOS,
        }),
    };

    pub const INITIAL_WSOL: Self = Self {
        mint: WSOL_MINT,
        nanos: FeeEntryNanos::const_from_destr(FeeEntryNanosDestr {
            base_fee: ZERO_FEE_NANOS,
            threshold: INITIAL_THRESHOLD_NANOS,
            threshold_fee: ZERO_FEE_NANOS,
            max_fee: ZERO_FEE_NANOS,
            output_fee: ZERO_FEE_NANOS,
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

pub const INITIAL_ENTRIES: [FeeEntry; 2] =
    if bytes_lt(&FeeEntry::INITIAL_LP.mint, &FeeEntry::INITIAL_WSOL.mint) {
        [FeeEntry::INITIAL_LP, FeeEntry::INITIAL_WSOL]
    } else {
        [FeeEntry::INITIAL_WSOL, FeeEntry::INITIAL_LP]
    };

const _ASSERT_INITIAL_ENTRIES_SORTED: () =
    assert!(bytes_lt(&INITIAL_ENTRIES[0].mint, &INITIAL_ENTRIES[1].mint));

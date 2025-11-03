// This does not seem to produce different bytecode
// on-chain compared to .copy_from_slice(), but it allows us to retain `const`
/// caba = `const_assign_byte_array`
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

/// const_split_byte_array - Splits an array reference of length `M` into two fixed-size slices:
/// - The first of length `N`
/// - The second of length `X`
///
/// # Type Parameters
/// - `M`: Total length of the input array.
/// - `N`: Length of the first returned slice.
/// - `X`: Length of the second returned slice (`X = M - N`).
///
/// # Panics (const-assert)
/// - If `N > M`
/// - If `X != M - N`
///
/// # Safety
/// Converts slice pointers into array references via raw pointers. This is safe
/// as long as the provided const parameters satisfy the assertions above.
///
/// # Example
/// ```
/// const DATA: [u8; 4] = [1, 2, 3, 4];
/// let (a, b) = split::<4, 2, 2>(&DATA);
/// assert_eq!(a, &[1, 2]);
/// assert_eq!(b, &[3, 4]);
/// ```
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

const DISCM_ONLY_IX_DATA_LEN: usize = 1;

/// Many admin-facing instructions take no additional instruction args
/// apart from the ix discm. This type generalizes their IxData type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DiscmOnlyIxData<const DISCM: u8>;

impl<const DISCM: u8> DiscmOnlyIxData<DISCM> {
    pub const DATA: u8 = DISCM;
    pub const DATA_LEN: usize = DISCM_ONLY_IX_DATA_LEN;

    #[inline]
    pub const fn as_buf() -> &'static [u8; DISCM_ONLY_IX_DATA_LEN] {
        &[Self::DATA]
    }
}

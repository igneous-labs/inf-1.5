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

// borsh format ser/de utils

macro_rules! deser_borsh_opt {
    ($data:expr, $parse_arr_fn:expr) => {{
        match $data.split_first() {
            Some((&1, rem)) => match rem.split_first_chunk() {
                Some((x, rem)) => Some((Some($parse_arr_fn(*x)), rem)),
                None => None,
            },
            Some((&0, rem)) => Some((None, rem)),
            _invalid => None,
        }
    }};
}

macro_rules! ser_borsh_opt_le_prim {
    ($data:expr, $opt:expr, $T:ty) => {{
        match ($opt, $data.split_first_mut()) {
            (None, Some((d, rem))) => {
                *d = 0;
                Some(rem)
            }
            (Some(val), Some((d, rem))) => {
                const N: usize = core::mem::size_of::<$T>();
                *d = 1;
                match rem.split_first_chunk_mut::<N>() {
                    Some((to, rem)) => {
                        *to = val.to_le_bytes();
                        Some(rem)
                    }
                    None => None,
                }
            }
            (_any, None) => None,
        }
    }};
}

/// Returns `(deserialized, remaining_data)`
///
/// Returns `Err` if data invalid
/// - option discriminant neither 1 nor 0
/// - data len not long enough
#[inline]
pub(crate) const fn deser_borsh_opt_u16(data: &[u8]) -> Option<(Option<u16>, &[u8])> {
    deser_borsh_opt!(data, u16::from_le_bytes)
}

/// Returns `remaining_bytes_after_newly_serialized_opt``
#[inline]
pub(crate) const fn ser_borsh_opt_u16(data: &mut [u8], opt: Option<u16>) -> Option<&mut [u8]> {
    ser_borsh_opt_le_prim!(data, opt, u16)
}

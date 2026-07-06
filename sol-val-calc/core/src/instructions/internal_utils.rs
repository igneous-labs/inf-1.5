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

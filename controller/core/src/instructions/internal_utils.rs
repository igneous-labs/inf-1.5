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

#[inline]
pub(crate) const fn split<T, const M: usize, const N: usize, const X: usize>(
    data: &[T; M],
) -> (&[T; N], &[T; X]) {
    const {
        assert!(N <= M);
        assert!(X == M - N)
    }

    let (a, b) = data.split_at(N);

    // SAFETY: data is guaranteed to be of length M
    // and we are splitting it into two slices of length N and X (i.e M-N)
    (unsafe { &*a.as_ptr().cast::<[T; N]>() }, unsafe {
        &*b.as_ptr().cast::<[T; X]>()
    })
}

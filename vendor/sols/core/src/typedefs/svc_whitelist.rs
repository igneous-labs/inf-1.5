use crate::{
    err::SortedListNoAddrErr,
    typedefs::{PkedList, PkedListMut},
};

/// Sorted ascending
pub type SvcWhitelist<'a> = PkedList<'a, [u8; 32]>;
pub type SvcWhitelistMut<'a> = PkedListMut<'a, [u8; 32]>;

#[inline]
pub fn search_svc_prog<'a>(
    svc_whitelist: &'a [[u8; 32]],
    svc_prog: &[u8; 32],
) -> Result<(usize, &'a [u8; 32]), SortedListNoAddrErr> {
    svc_whitelist
        .binary_search_by_key(svc_prog, |a| *a)
        .map_err(|idx| SortedListNoAddrErr {
            addr: *svc_prog,
            idx,
        })
        .map(|i| (i, &svc_whitelist[i]))
}

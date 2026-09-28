use core::borrow::Borrow;

use crate::{
    err::SortedListNoAddrErr,
    typedefs::{
        HoldingV1, HoldingV1AddrVals, HoldingV1Entry, HoldingV1LamportVals, HoldingV1Lamports,
        HoldingV1LamportsDestr, HoldingV1Pked, PkedList, PkedListMut,
    },
};

/// Sorted by mint ascending
pub type PortfolioV1<'a> = PkedList<'a, HoldingV1Entry>;
pub type PortfolioV1Mut<'a> = PkedListMut<'a, HoldingV1Entry>;
pub type PortfolioV1Pked<'a> = PkedList<'a, HoldingV1Pked>;
pub type PortfolioV1PkedMut<'a> = PkedListMut<'a, HoldingV1Pked>;

#[allow(clippy::type_complexity)]
#[inline]
pub fn search_holding<'a, L, N, F, B, P>(
    portfolio: &'a [HoldingV1<HoldingV1AddrVals, L, N, F, B, P>],
    mint: &[u8; 32],
) -> Result<(usize, &'a HoldingV1<HoldingV1AddrVals, L, N, F, B, P>), SortedListNoAddrErr> {
    portfolio
        .binary_search_by_key(mint, |h| *h.addrs.mint())
        .map_err(|idx| SortedListNoAddrErr { addr: *mint, idx })
        .map(|i| (i, &portfolio[i]))
}

#[allow(clippy::type_complexity)]
#[inline]
pub fn search_holding_mut<'a, L, N, F, B, P>(
    portfolio: &'a mut [HoldingV1<HoldingV1AddrVals, L, N, F, B, P>],
    mint: &[u8; 32],
) -> Result<(usize, &'a mut HoldingV1<HoldingV1AddrVals, L, N, F, B, P>), SortedListNoAddrErr> {
    portfolio
        .binary_search_by_key(mint, |h| *h.addrs.mint())
        .map_err(|idx| SortedListNoAddrErr { addr: *mint, idx })
        .map(|i| (i, &mut portfolio[i]))
}

/// Returns `None` on overflow
#[inline]
pub fn sum_holding_lamports(
    holding_lamports_itr: impl IntoIterator<Item = impl Borrow<HoldingV1LamportVals>>,
) -> Option<HoldingV1LamportVals> {
    holding_lamports_itr.into_iter().try_fold(
        HoldingV1Lamports::const_from_destr(HoldingV1LamportsDestr {
            outstanding: 0u64,
            sol_value: 0u64,
        }),
        |acc, h| {
            let h = h.borrow();
            Some(
                acc.with_outstanding(acc.outstanding().checked_add(*h.outstanding())?)
                    .with_sol_value(acc.sol_value().checked_add(*h.sol_value())?),
            )
        },
    )
}

/// Returns `None` on overflow
#[inline]
pub fn sum_portfolio_lamports<A, N, F, B, P>(
    portfolio: &[HoldingV1<A, HoldingV1LamportVals, N, F, B, P>],
) -> Option<HoldingV1LamportVals> {
    sum_holding_lamports(portfolio.iter().map(|h| h.lamports))
}

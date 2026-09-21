use core::{error::Error, fmt::Display};

use inf1_pp_core::{
    instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
    traits::main::{PriceExactIn, PriceExactOut},
};

#[allow(deprecated)]
use inf1_pp_core::{
    instructions::deprecated::lp::{
        mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs,
    },
    traits::deprecated::{PriceLpTokensToMint, PriceLpTokensToRedeem},
};

use crate::typedefs::FeeEntry;

mod flat;
mod range;
mod retained;

pub use flat::FlatPricing;
pub use range::{range_out_inputs, InputFeeCurve, RangeOutInputsErr, RangeOutPricing};

/// The pricing applied to one swap, discriminated by route.
///
/// The reserve-v2 pricing program resolves every swap to one of two shapes via
/// [`classify_route`](crate::route::classify_route): a flat fee
/// ([`FlatPricing`]) or a range/over-the-cap fee that depends on the pool's SOL
/// value and wSOL reserves ([`RangeOutPricing`])
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReserveV2Swap<Flat, RangeOut> {
    Flat(Flat),
    RangeOut(RangeOut),
}

/// The pricing of a reserve-v2 swap: [`ReserveV2Swap`] with its fee shapes
/// filled in.
pub type ReserveV2SwapPricing = ReserveV2Swap<FlatPricing, RangeOutPricing>;

impl ReserveV2Swap<FlatPricing, RangeOutPricing> {
    /// Fill in the concrete pricing for a
    /// [`ReserveV2SwapKind`](crate::route::ReserveV2SwapKind).
    ///
    /// `pool_sol_value` and `wsol_balance` are only read by the `RangeOut` arm;
    /// they are ignored by `Flat`. Callers that resolve a flat route do not
    /// need to fetch them.
    #[inline]
    pub const fn from_entries(
        route: crate::route::ReserveV2SwapKind,
        input_entry: &FeeEntry,
        output_entry: &FeeEntry,
        pool_sol_value: u64,
        wsol_balance: u64,
    ) -> Self {
        match route {
            crate::route::ReserveV2SwapKind::Flat(()) => {
                Self::Flat(FlatPricing::from_entries(input_entry, output_entry))
            }
            crate::route::ReserveV2SwapKind::RangeOut(()) => {
                Self::RangeOut(RangeOutPricing::from_entries(
                    input_entry,
                    output_entry,
                    pool_sol_value,
                    wsol_balance,
                ))
            }
        }
    }
}

// Display + Error blanket, so the aggregate error folds like `PricingAg`'s

impl<Flat: Display, RangeOut: Display> Display for ReserveV2Swap<Flat, RangeOut> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Flat(p) => Display::fmt(p, f),
            Self::RangeOut(p) => Display::fmt(p, f),
        }
    }
}

impl<Flat: Error, RangeOut: Error> Error for ReserveV2Swap<Flat, RangeOut> {}

// Quoting

impl<Flat: PriceExactIn, RangeOut: PriceExactIn> PriceExactIn for ReserveV2Swap<Flat, RangeOut> {
    type Error = ReserveV2Swap<Flat::Error, RangeOut::Error>;

    #[inline]
    fn price_exact_in(&self, args: PriceExactInIxArgs) -> Result<u64, Self::Error> {
        match self {
            Self::Flat(p) => PriceExactIn::price_exact_in(p, args).map_err(ReserveV2Swap::Flat),
            Self::RangeOut(p) => {
                PriceExactIn::price_exact_in(p, args).map_err(ReserveV2Swap::RangeOut)
            }
        }
    }
}

impl<Flat: PriceExactOut, RangeOut: PriceExactOut> PriceExactOut for ReserveV2Swap<Flat, RangeOut> {
    type Error = ReserveV2Swap<Flat::Error, RangeOut::Error>;

    #[inline]
    fn price_exact_out(&self, args: PriceExactOutIxArgs) -> Result<u64, Self::Error> {
        match self {
            Self::Flat(p) => PriceExactOut::price_exact_out(p, args).map_err(ReserveV2Swap::Flat),
            Self::RangeOut(p) => {
                PriceExactOut::price_exact_out(p, args).map_err(ReserveV2Swap::RangeOut)
            }
        }
    }
}

// The reserve-v2 pricing program rejects the deprecated LP instructions
// outright; the aggregate still has to name a type for them.

#[allow(deprecated)]
impl<Flat: PriceLpTokensToMint, RangeOut: PriceLpTokensToMint> PriceLpTokensToMint
    for ReserveV2Swap<Flat, RangeOut>
{
    type Error = ReserveV2Swap<Flat::Error, RangeOut::Error>;

    #[inline]
    fn price_lp_tokens_to_mint(&self, args: PriceLpTokensToMintIxArgs) -> Result<u64, Self::Error> {
        match self {
            Self::Flat(p) => {
                PriceLpTokensToMint::price_lp_tokens_to_mint(p, args).map_err(ReserveV2Swap::Flat)
            }
            Self::RangeOut(p) => PriceLpTokensToMint::price_lp_tokens_to_mint(p, args)
                .map_err(ReserveV2Swap::RangeOut),
        }
    }
}

#[allow(deprecated)]
impl<Flat: PriceLpTokensToRedeem, RangeOut: PriceLpTokensToRedeem> PriceLpTokensToRedeem
    for ReserveV2Swap<Flat, RangeOut>
{
    type Error = ReserveV2Swap<Flat::Error, RangeOut::Error>;

    #[inline]
    fn price_lp_tokens_to_redeem(
        &self,
        args: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        match self {
            Self::Flat(p) => PriceLpTokensToRedeem::price_lp_tokens_to_redeem(p, args)
                .map_err(ReserveV2Swap::Flat),
            Self::RangeOut(p) => PriceLpTokensToRedeem::price_lp_tokens_to_redeem(p, args)
                .map_err(ReserveV2Swap::RangeOut),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        route::ReserveV2SwapKind,
        typedefs::{FeeEntryNanos, FeeEntryNanosDestr},
    };

    use super::*;

    fn entry(mint: [u8; 32], base: u32, output: u32) -> FeeEntry {
        FeeEntry {
            mint,
            threshold_nanos: 1,
            fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
                base_fee: base,
                threshold_fee: base,
                max_fee: base,
                output_fee: output,
            }),
        }
    }

    #[test]
    fn flat_route_ignores_pool_state() {
        let inp = entry([1; 32], 1_000_000, 2_000_000);
        let out = entry([2; 32], 3_000_000, 4_000_000);
        let a = ReserveV2Swap::from_entries(ReserveV2SwapKind::Flat(()), &inp, &out, 0, 0);
        let b = ReserveV2Swap::from_entries(ReserveV2SwapKind::Flat(()), &inp, &out, 999, 123);
        assert_eq!(a, b);
        assert!(matches!(a, ReserveV2Swap::Flat(_)));
    }

    #[test]
    fn range_route_carries_pool_state() {
        let inp = entry([1; 32], 1_000_000, 2_000_000);
        let out = entry([2; 32], 3_000_000, 4_000_000);
        let p = ReserveV2Swap::from_entries(ReserveV2SwapKind::RangeOut(()), &inp, &out, 500, 400);
        let ReserveV2Swap::RangeOut(r) = p else {
            panic!("expected range pricing");
        };
        assert_eq!(r.pool_sol_value, 500);
        assert_eq!(r.wsol_balance, 400);
    }

    #[test]
    fn aggregates_quote_errors() {
        let inp = entry([1; 32], 0, 0);
        let out = entry([2; 32], 0, 0);
        let p = ReserveV2Swap::from_entries(ReserveV2SwapKind::Flat(()), &inp, &out, 0, 0);
        let got = p
            .price_exact_in(PriceExactInIxArgs {
                amt: 0,
                sol_value: 1_000,
            })
            .unwrap();
        assert_eq!(got, 1_000);
    }
}

use core::{error::Error, fmt::Display};

use inf1_pp_core::{
    instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
    traits::main::{PriceExactIn, PriceExactOut},
};
use sanctum_u64_ratio::{Floor, Ratio};

#[allow(deprecated)]
use inf1_pp_core::{
    instructions::deprecated::lp::{
        mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs,
    },
    traits::deprecated::{PriceLpTokensToMint, PriceLpTokensToRedeem},
};

use crate::typedefs::{FeeNanos, NANOS_DENOM};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlatSlabSwapPricing {
    /// Read from [`crate::accounts::SlabEntryPacked::inp_fee_nanos`] of input LST.
    ///
    /// Should be that of [`crate::keys::LP_MINT_ID`] for RemoveLiquidity
    pub inp_fee_nanos: FeeNanos,

    /// Read from [`crate::accounts::SlabEntryPacked::out_fee_nanos`] of output LST
    ///
    /// Should be that of [`crate::keys::LP_MINT_ID`] for AddLiquidity
    pub out_fee_nanos: FeeNanos,
}

/// Checks
impl FlatSlabSwapPricing {
    #[inline]
    pub const fn is_net_negative(&self) -> bool {
        // unchecked-arith: FeeNanos valid range will not overflow
        self.inp_fee_nanos.get() + self.out_fee_nanos.get() < 0
    }
}

/// Pricing
impl FlatSlabSwapPricing {
    /// Returns the ratio that returns out_sol_value
    /// when applied to in_sol_value
    ///
    /// Returns None if self's data result in overflow
    #[inline]
    pub const fn out_ratio(&self) -> Option<Floor<Ratio<u32, u32>>> {
        let fee_nanos = match self
            .inp_fee_nanos
            .get()
            .checked_add(self.out_fee_nanos.get())
        {
            None => return None,
            Some(f) => f,
        };
        // post_fee_nanos = 1_000_000_000 - fee_nanos
        // out_sol_value = floor(in_sol_value * post_fee_nanos / 1_000_000_000)
        // i32 signed subtraction:
        // - rebates are allowed (post_fee_nanos > 1_000_000_000)
        // - however, >100% fees will error (post_fee_nanos < 0)
        let post_fee_nanos = match NANOS_DENOM.checked_sub(fee_nanos) {
            None => return None,
            Some(f) => f,
        };
        let post_fee_nanos = if post_fee_nanos < 0 {
            return None;
        } else {
            post_fee_nanos as u32
        };
        Some(Floor(Ratio {
            n: post_fee_nanos,
            d: NANOS_DENOM as u32,
        }))
    }

    #[inline]
    pub const fn pp_price_exact_in(&self, in_sol_value: u64) -> Option<u64> {
        match self.out_ratio() {
            None => None,
            Some(r) => r.apply(in_sol_value),
        }
    }

    #[inline]
    pub const fn pp_price_exact_out(&self, out_sol_value: u64) -> Option<u64> {
        // the greatest possible non-u64::MAX value of in_sol_value is 1_000_000_00 x out_sol_value.
        // Otherwise if fee is 100% then this will return None unless out_sol_value == 0
        let range_opt = match self.out_ratio() {
            None => return None,
            Some(r) => r.reverse(out_sol_value),
        };
        let range = match range_opt {
            None => return None,
            Some(r) => r,
        };
        Some(*range.end())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatSlabPricingErr {
    NetNegativeFees,
    Ratio,
}

impl Display for FlatSlabPricingErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NetNegativeFees => "net negative fees disallowed",
            Self::Ratio => "ratio math error",
        })
    }
}

impl Error for FlatSlabPricingErr {}

impl PriceExactIn for FlatSlabSwapPricing {
    type Error = FlatSlabPricingErr;

    #[inline]
    fn price_exact_in(
        &self,
        PriceExactInIxArgs { sol_value, .. }: PriceExactInIxArgs,
    ) -> Result<u64, Self::Error> {
        if self.is_net_negative() {
            return Err(FlatSlabPricingErr::NetNegativeFees);
        }
        self.pp_price_exact_in(sol_value)
            .ok_or(FlatSlabPricingErr::Ratio)
    }
}

impl PriceExactOut for FlatSlabSwapPricing {
    type Error = FlatSlabPricingErr;

    #[inline]
    fn price_exact_out(
        &self,
        PriceExactOutIxArgs { sol_value, .. }: PriceExactOutIxArgs,
    ) -> Result<u64, Self::Error> {
        if self.is_net_negative() {
            return Err(FlatSlabPricingErr::NetNegativeFees);
        }
        self.pp_price_exact_out(sol_value)
            .ok_or(FlatSlabPricingErr::Ratio)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToRedeem for FlatSlabSwapPricing {
    type Error = FlatSlabPricingErr;

    #[inline]
    fn price_lp_tokens_to_redeem(
        &self,
        PriceLpTokensToRedeemIxArgs { sol_value, .. }: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        if self.is_net_negative() {
            return Err(FlatSlabPricingErr::NetNegativeFees);
        }
        self.pp_price_exact_in(sol_value)
            .ok_or(FlatSlabPricingErr::Ratio)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToMint for FlatSlabSwapPricing {
    type Error = FlatSlabPricingErr;

    fn price_lp_tokens_to_mint(
        &self,
        PriceLpTokensToMintIxArgs { sol_value, .. }: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        if self.is_net_negative() {
            return Err(FlatSlabPricingErr::NetNegativeFees);
        }
        self.pp_price_exact_in(sol_value)
            .ok_or(FlatSlabPricingErr::Ratio)
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::{max, min};

    use proptest::prelude::*;

    use super::*;

    #[test]
    fn basic() {
        // 1bps
        let p = FlatSlabSwapPricing {
            inp_fee_nanos: FeeNanos::new(100_000).unwrap(),
            out_fee_nanos: FeeNanos::new(0).unwrap(),
        };
        let sol_value = 999_999_999;
        let amt = 0; // dont care

        let exact_in = p
            .price_exact_in(PriceExactInIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(exact_in, 999_899_999);

        #[allow(deprecated)]
        let mint = p
            .price_lp_tokens_to_mint(PriceLpTokensToMintIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(mint, exact_in);

        #[allow(deprecated)]
        let redeem = p
            .price_lp_tokens_to_redeem(PriceLpTokensToRedeemIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(redeem, exact_in);

        assert_eq!(
            p.price_exact_out(PriceExactInIxArgs { amt, sol_value })
                .unwrap(),
            1_000_100_010,
        );
    }

    // proptests

    fn fees_same_sum() -> impl Strategy<Value = [FlatSlabSwapPricing; 2]> {
        (*FeeNanos::MIN..=*FeeNanos::MAX)
            .prop_flat_map(|i1| {
                (
                    Just(i1),
                    if i1 < 0 {
                        *FeeNanos::MIN - i1..=*FeeNanos::MAX
                    } else {
                        *FeeNanos::MIN..=*FeeNanos::MAX - i1
                    },
                )
            })
            .prop_flat_map(|(i1, o1)| {
                let sum = i1 + o1;
                (
                    Just(FeeNanos::new(i1).unwrap()),
                    Just(FeeNanos::new(o1).unwrap()),
                    max(*FeeNanos::MIN, sum - *FeeNanos::MAX)
                        ..=min(*FeeNanos::MAX, sum - *FeeNanos::MIN),
                    Just(sum),
                )
            })
            .prop_map(|(i1, o1, i2, sum)| {
                [
                    FlatSlabSwapPricing {
                        inp_fee_nanos: i1,
                        out_fee_nanos: o1,
                    },
                    FlatSlabSwapPricing {
                        inp_fee_nanos: FeeNanos::new(i2).unwrap(),
                        out_fee_nanos: FeeNanos::new(sum - i2).unwrap(),
                    },
                ]
            })
    }

    prop_compose! {
        /// inp out nanos pair that will result in a fee rate in [0, 1.0]
        fn zero_incl_one_incl_fee()
            (inp_fee_nanos in *FeeNanos::MIN..=*FeeNanos::MAX)
            (
                o in max(*FeeNanos::MIN, -inp_fee_nanos)..=min(*FeeNanos::MAX, NANOS_DENOM - inp_fee_nanos),
                i in Just(inp_fee_nanos)
            ) -> FlatSlabSwapPricing {
                FlatSlabSwapPricing {
                    inp_fee_nanos: FeeNanos::new(i).unwrap(),
                    out_fee_nanos: FeeNanos::new(o).unwrap()
                }
            }
    }

    prop_compose! {
        /// inp out nanos pair that will result in a fee rate in (0, 1.0]
        fn zero_excl_one_incl_fee()
            (inp_fee_nanos in *FeeNanos::MIN..=*FeeNanos::MAX)
            (
                o in max(*FeeNanos::MIN, 1 - inp_fee_nanos)..=min(*FeeNanos::MAX, NANOS_DENOM - inp_fee_nanos),
                i in Just(inp_fee_nanos)
            ) -> FlatSlabSwapPricing {
                FlatSlabSwapPricing {
                    inp_fee_nanos: FeeNanos::new(i).unwrap(),
                    out_fee_nanos: FeeNanos::new(o).unwrap(),
                }
            }
    }

    prop_compose! {
        /// inp out nanos pair that will result in a fee of 0
        fn zero_fee()
            (i in *FeeNanos::MIN..=-*FeeNanos::MIN) -> FlatSlabSwapPricing {
                // ensure we've use the boundary val with smaller abs value to avoid out of range
                assert!(FeeNanos::MIN.abs() <= FeeNanos::MAX.abs());
                FlatSlabSwapPricing {
                    inp_fee_nanos: FeeNanos::new(i).unwrap(),
                    out_fee_nanos: FeeNanos::new(-i).unwrap(),
                }
            }
    }

    prop_compose! {
        /// inp out nanos pair that will result in a fee rate of 1.0
        fn one_fee()
            (i in 0..=*FeeNanos::MAX) // must be >= 0 to achieve 1.0 due to MAX = NANOS_DENOM
            -> FlatSlabSwapPricing {
                assert!(*FeeNanos::MAX == NANOS_DENOM);
                FlatSlabSwapPricing {
                    inp_fee_nanos: FeeNanos::new(i).unwrap(),
                    out_fee_nanos: FeeNanos::new(NANOS_DENOM - i).unwrap()
                }
            }
    }

    prop_compose! {
        /// inp out nanos pair that will result in a fee rate < 0
        fn net_neg_fee()
            (i in *FeeNanos::MIN..=-*FeeNanos::MIN) // since abs(MIN) < abs(MAX)
            (i in Just(i), o in *FeeNanos::MIN..-i)
            -> FlatSlabSwapPricing {
                FlatSlabSwapPricing {
                    inp_fee_nanos: FeeNanos::new(i).unwrap(),
                    out_fee_nanos: FeeNanos::new(o).unwrap()
                }
            }
    }

    // General

    proptest! {
        #[test]
        fn same_sum_of_fields_should_have_same_behaviour(
            [f1, f2] in fees_same_sum(),
            sol_value: u64,
            amt: u64,
        ) {
            let args = PriceExactInIxArgs {
                sol_value,
                amt,
            };
            for pf in [
                FlatSlabSwapPricing::price_exact_in,
                FlatSlabSwapPricing::price_exact_out,
                #[allow(deprecated)]
                FlatSlabSwapPricing::price_lp_tokens_to_mint,
                #[allow(deprecated)]
                FlatSlabSwapPricing::price_lp_tokens_to_redeem,
            ] {
                match (pf(&f1, args), pf(&f2, args)) {
                    (Ok(a), Ok(b)) => prop_assert_eq!(a, b),
                    (Err(a), Err(b)) => prop_assert_eq!(a, b),
                    (a, b) => panic!("{a:#?}, {b:#?}")
                }
            }
        }
    }

    // ExactIn

    proptest! {
        #[test]
        fn zioi_fee_exact_in_gives_lte_out_sol_value(
            fee in zero_incl_one_incl_fee(),
            in_sol_value: u64,
            amt: u64, // dont-care
        ) {
            let out_sol_value = fee.price_exact_in(
                PriceExactInIxArgs { sol_value: in_sol_value, amt }
            ).unwrap();
            prop_assert!(out_sol_value <= in_sol_value);
        }
    }

    proptest! {
        #[test]
        fn zeoi_fee_exact_in_gives_lt_out_sol_value(
            fee in zero_excl_one_incl_fee(),
            in_sol_value: u64,
            amt: u64, // dont-care
        ) {
            let out_sol_value = fee.price_exact_in(
                PriceExactInIxArgs { sol_value: in_sol_value, amt }
            ).unwrap();
            prop_assert!(out_sol_value < in_sol_value);
        }
    }

    proptest! {
        #[test]
        fn zero_fee_exact_in_gives_same_sol_value(
            fee in zero_fee(),
            in_sol_value: u64,
            amt: u64, // dont-care
        ) {
            let out_sol_value = fee.price_exact_in(
                PriceExactInIxArgs { sol_value: in_sol_value, amt }
            ).unwrap();
            prop_assert_eq!(out_sol_value, in_sol_value);
        }
    }

    proptest! {
        #[test]
        fn one_fee_exact_in_gives_zero_sol_value(
            fee in one_fee(),
            in_sol_value: u64,
            amt: u64, // dont-care
        ) {
            let out_sol_value = fee.price_exact_in(
                PriceExactInIxArgs { sol_value: in_sol_value, amt }
            ).unwrap();
            prop_assert_eq!(out_sol_value, 0);
        }
    }

    // ExactOut

    proptest! {
        #[test]
        fn zioi_fee_exact_out_gives_gte_in_sol_value(
            fee in zero_incl_one_incl_fee(),
            out_sol_value in 0..=(u64::MAX / NANOS_DENOM as u64),
            amt: u64, // dont-care
        ) {
            let in_sol_value = fee.price_exact_out(
                PriceExactInIxArgs { sol_value: out_sol_value, amt }
            ).unwrap();
            prop_assert!(out_sol_value <= in_sol_value);
        }
    }

    proptest! {
        #[test]
        fn zeoi_fee_exact_out_gives_gt_in_sol_value(
            fee in zero_excl_one_incl_fee(),
            out_sol_value in 0..=(u64::MAX / NANOS_DENOM as u64),
            amt: u64, // dont-care
        ) {
            let in_sol_value = fee.price_exact_out(
                PriceExactInIxArgs { sol_value: out_sol_value, amt }
            ).unwrap();
            prop_assert!(out_sol_value < in_sol_value);
        }
    }

    proptest! {
        #[test]
        fn zero_fee_exact_out_gives_same_sol_value(
            fee in zero_fee(),
            out_sol_value: u64,
            amt: u64, // dont-care
        ) {
            let in_sol_value = fee.price_exact_out(
                PriceExactInIxArgs { sol_value: out_sol_value, amt }
            ).unwrap();
            prop_assert_eq!(out_sol_value, in_sol_value);
        }
    }

    proptest! {
        #[test]
        fn one_fee_exact_out_errs_if_out_sol_value_nonzero(
            fee in one_fee(),
            out_sol_value in 1..=(u64::MAX / NANOS_DENOM as u64),
            amt: u64, // dont-care
        ) {
            let res = fee.price_exact_out(
                PriceExactInIxArgs { sol_value: out_sol_value, amt }
            );
            prop_assert!(res.is_err());
        }
    }

    proptest! {
        #[test]
        fn one_fee_exact_out_max_if_out_sol_value_zero(
            fee in one_fee(),
            out_sol_value in Just(0),
            amt: u64, // dont-care
        ) {
            let in_sol_value = fee.price_exact_out(
                PriceExactInIxArgs { sol_value: out_sol_value, amt }
            ).unwrap();
            prop_assert_eq!(in_sol_value, u64::MAX);
        }
    }

    // MintLp

    proptest! {
        #[test]
        fn zioi_fee_mint_lp_gives_lte_mint_sol_value(
            fee in zero_incl_one_incl_fee(),
            sol_value: u64,
            amt: u64, // dont-care
        ) {
            #[allow(deprecated)]
            let mint_sol_value = fee.price_lp_tokens_to_mint(
                PriceLpTokensToMintIxArgs { sol_value, amt }
            ).unwrap();
            prop_assert!(mint_sol_value <= sol_value);
        }
    }

    proptest! {
        #[test]
        fn zeoi_fee_mint_lp_gives_lt_mint_sol_value(
            fee in zero_excl_one_incl_fee(),
            sol_value: u64,
            amt: u64, // dont-care
        ) {
            #[allow(deprecated)]
            let mint_sol_value = fee.price_lp_tokens_to_mint(
                PriceLpTokensToMintIxArgs { sol_value, amt }
            ).unwrap();
            prop_assert!(mint_sol_value < sol_value);
        }
    }

    proptest! {
        #[test]
        fn zero_fee_mint_lp_gives_same_sol_value(
            fee in zero_fee(),
            sol_value: u64,
            amt: u64, // dont-care
        ) {
            #[allow(deprecated)]
            let mint_sol_value = fee.price_lp_tokens_to_mint(
                PriceExactInIxArgs { sol_value, amt }
            ).unwrap();
            prop_assert_eq!(mint_sol_value, sol_value);
        }
    }

    // RedeemLp

    proptest! {
        #[test]
        fn zioi_fee_redeem_lp_gives_lte_redeem_sol_value(
            fee in zero_incl_one_incl_fee(),
            sol_value: u64,
            amt: u64, // dont-care
        ) {
            #[allow(deprecated)]
            let redeem_sol_value = fee.price_lp_tokens_to_redeem(
                PriceLpTokensToRedeemIxArgs { sol_value, amt }
            ).unwrap();
            prop_assert!(redeem_sol_value <= sol_value);
        }
    }

    proptest! {
        #[test]
        fn zeoi_fee_redeem_lp_gives_lt_redeem_sol_value(
            fee in zero_excl_one_incl_fee(),
            sol_value: u64,
            amt: u64, // dont-care
        ) {
            #[allow(deprecated)]
            let redeem_sol_value = fee.price_lp_tokens_to_redeem(
                PriceLpTokensToRedeemIxArgs { sol_value, amt }
            ).unwrap();
            prop_assert!(redeem_sol_value < sol_value);
        }
    }

    proptest! {
        #[test]
        fn zero_fee_redeem_lp_gives_same_sol_value(
            fee in zero_fee(),
            sol_value: u64,
            amt: u64, // dont-care
        ) {
            #[allow(deprecated)]
            let mint_sol_value = fee.price_lp_tokens_to_redeem(
                PriceExactInIxArgs { sol_value, amt }
            ).unwrap();
            prop_assert_eq!(mint_sol_value, sol_value);
        }
    }

    proptest! {
        #[allow(deprecated)]
        #[test]
        fn net_neg_fee_errs(
            fee in net_neg_fee(),
            sol_value: u64,
            amt: u64, // dont-care
        ) {
            let args = PriceExactInIxArgs { sol_value, amt };
            for f in [
                FlatSlabSwapPricing::price_exact_in,
                FlatSlabSwapPricing::price_exact_out,
                FlatSlabSwapPricing::price_lp_tokens_to_redeem,
                FlatSlabSwapPricing::price_lp_tokens_to_mint,
            ] {
                prop_assert_eq!(
                    f(&fee, args),
                    Err(FlatSlabPricingErr::NetNegativeFees),
                );
            }
        }
    }
}

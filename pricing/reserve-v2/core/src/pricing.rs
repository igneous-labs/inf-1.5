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

use crate::{
    errs::ReserveV2ProgramErr,
    typedefs::{FeeEntryPacked, FeeNanos, NANOS_DENOM},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlatPricing {
    pub input_fee_nanos: FeeNanos,
    pub output_fee_nanos: FeeNanos,
}

impl FlatPricing {
    #[inline]
    pub const fn from_entries(input_entry: &FeeEntryPacked, output_entry: &FeeEntryPacked) -> Self {
        Self {
            input_fee_nanos: input_entry.base_fee_nanos(),
            output_fee_nanos: output_entry.output_fee_nanos(),
        }
    }

    #[inline]
    pub fn pp_price_exact_in(&self, input_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        price_exact_in_retained_product(
            input_sol_value,
            self.input_fee_nanos,
            self.output_fee_nanos,
        )
    }

    #[inline]
    pub fn pp_price_exact_out(&self, output_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        price_exact_out_retained_product(
            output_sol_value,
            self.input_fee_nanos,
            self.output_fee_nanos,
        )
    }
}

impl PriceExactIn for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_in(
        &self,
        PriceExactInIxArgs { sol_value, .. }: PriceExactInIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_in(sol_value)
    }
}

impl PriceExactOut for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_out(
        &self,
        PriceExactOutIxArgs { sol_value, .. }: PriceExactOutIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_out(sol_value)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToMint for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_lp_tokens_to_mint(
        &self,
        _input: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToRedeem for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_lp_tokens_to_redeem(
        &self,
        _input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
    }
}

#[inline]
fn price_exact_in_retained_product(
    input_sol_value: u64,
    input_fee_nanos: FeeNanos,
    output_fee_nanos: FeeNanos,
) -> Result<u64, ReserveV2ProgramErr> {
    let ratio = route_retained_ratio(input_fee_nanos, output_fee_nanos)?;
    ratio
        .apply(input_sol_value)
        .ok_or(ReserveV2ProgramErr::MathOverflow)
}

#[inline]
fn price_exact_out_retained_product(
    output_sol_value: u64,
    input_fee_nanos: FeeNanos,
    output_fee_nanos: FeeNanos,
) -> Result<u64, ReserveV2ProgramErr> {
    let ratio = route_retained_ratio(input_fee_nanos, output_fee_nanos)?;
    let range = ratio
        .reverse(output_sol_value)
        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
    Ok(*range.start())
}

#[inline]
fn route_retained_ratio(
    input_fee_nanos: FeeNanos,
    output_fee_nanos: FeeNanos,
) -> Result<Floor<Ratio<u64, u64>>, ReserveV2ProgramErr> {
    let input_retained_nanos = input_fee_nanos.retained() as u64;
    let output_retained_nanos = output_fee_nanos.retained() as u64;
    // retained values are <= NANOS_DENOM so product <= N^2, which fits in u64
    let retained_product = input_retained_nanos * output_retained_nanos;
    if retained_product == 0 {
        return Err(ReserveV2ProgramErr::ZeroRetainedValue);
    }
    let denom = (NANOS_DENOM as u64) * (NANOS_DENOM as u64);
    Ok(Floor(Ratio {
        n: retained_product,
        d: denom,
    }))
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn flat_pricing_basic() {
        let pricing = FlatPricing {
            input_fee_nanos: FeeNanos::new(500_000_000).unwrap(),
            output_fee_nanos: FeeNanos::new(600_000_000).unwrap(),
        };
        let amt = 0;
        let sol_value = 1_000;
        let exact_in = pricing
            .price_exact_in(PriceExactInIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(exact_in, 200);
        let exact_out = pricing
            .price_exact_out(PriceExactOutIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(exact_out, 5_000);
    }

    #[test]
    fn flat_pricing_exact_out_rounds_up() {
        // 1bps
        let pricing = FlatPricing {
            input_fee_nanos: FeeNanos::new(100_000).unwrap(),
            output_fee_nanos: FeeNanos::ZERO,
        };
        let amt = 0;
        let exact_in = pricing
            .price_exact_in(PriceExactInIxArgs {
                amt,
                sol_value: 999_999_999,
            })
            .unwrap();
        assert_eq!(exact_in, 999_899_999);
        // non-exact division: minimal sufficient input rounds up
        let exact_out = pricing
            .price_exact_out(PriceExactOutIxArgs {
                amt,
                sol_value: 999_899_999,
            })
            .unwrap();
        assert_eq!(exact_out, 999_999_999);
        // exact division: no rounding
        let exact_out = pricing
            .price_exact_out(PriceExactOutIxArgs {
                amt,
                sol_value: 999_900_000,
            })
            .unwrap();
        assert_eq!(exact_out, 1_000_000_000);
    }

    #[test]
    fn flat_pricing_zero_retained_value() {
        for pricing in [
            FlatPricing {
                input_fee_nanos: FeeNanos::MAX,
                output_fee_nanos: FeeNanos::ZERO,
            },
            FlatPricing {
                input_fee_nanos: FeeNanos::ZERO,
                output_fee_nanos: FeeNanos::MAX,
            },
        ] {
            let amt = 0;
            let sol_value = 1;
            let exact_in = pricing.price_exact_in(PriceExactInIxArgs { amt, sol_value });
            assert_eq!(exact_in, Err(ReserveV2ProgramErr::ZeroRetainedValue));
            let exact_out = pricing.price_exact_out(PriceExactOutIxArgs { amt, sol_value });
            assert_eq!(exact_out, Err(ReserveV2ProgramErr::ZeroRetainedValue));
        }
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_lp_instructions() {
        let pricing = FlatPricing {
            input_fee_nanos: FeeNanos::ZERO,
            output_fee_nanos: FeeNanos::ZERO,
        };
        let amt = 0;
        let sol_value = 1;
        let mint = pricing.price_lp_tokens_to_mint(PriceLpTokensToMintIxArgs { amt, sol_value });
        assert_eq!(
            mint,
            Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
        );
        let redeem =
            pricing.price_lp_tokens_to_redeem(PriceLpTokensToRedeemIxArgs { amt, sol_value });
        assert_eq!(
            redeem,
            Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
        );
    }

    fn nonzero_retained_fee_nanos() -> impl Strategy<Value = FeeNanos> {
        (0..NANOS_DENOM).prop_map(|n| FeeNanos::new(n).unwrap())
    }

    prop_compose! {
        fn flat_nonzero_retained()
            (
                input_fee_nanos in nonzero_retained_fee_nanos(),
                output_fee_nanos in nonzero_retained_fee_nanos(),
            ) -> FlatPricing {
                FlatPricing {
                    input_fee_nanos,
                    output_fee_nanos,
                }
            }
    }

    proptest! {
        #[test]
        fn flat_pricing_round_trip_and_minimum_sufficient_input(
            pricing in flat_nonzero_retained(),
            input_sol_value: u64,
            amt: u64,
        ) {
            let out_sol_value = pricing
                .price_exact_in(PriceExactInIxArgs {
                    amt,
                    sol_value: input_sol_value,
                })
                .unwrap();
            let required_sol_value = pricing
                .price_exact_out(PriceExactOutIxArgs {
                    amt,
                    sol_value: out_sol_value,
                })
                .unwrap();

            prop_assert!(required_sol_value <= input_sol_value);
            let round_trip_out = pricing
                .price_exact_in(PriceExactInIxArgs {
                    amt,
                    sol_value: required_sol_value,
                })
                .unwrap();
            prop_assert!(round_trip_out >= out_sol_value);
            if required_sol_value > 0 {
                let too_low_out = pricing
                    .price_exact_in(PriceExactInIxArgs {
                        amt,
                        sol_value: required_sol_value - 1,
                    })
                    .unwrap();
                prop_assert!(too_low_out < out_sol_value);
            }
        }
    }
}

use core::convert::Infallible;

use inf1_svc_core::traits::SolValCalc;
use sanctum_u64_ratio::{Ceil, Ratio};

use crate::{
    err::OutOfRangeErrU64,
    rebal::{QuoteRebalErr, RebalInpOut, RebalQtysDestr, RebalQuote, RebalQuoteSolVal},
    typedefs::HoldingV1LamportVals,
    utils::{InpOut, InpOutDestr},
};

#[inline]
pub fn quote_rebal_other_exact_out<I: SolValCalc>(
    out: u64,
    out_holding_balance: u64,
    out_holding: &HoldingV1LamportVals,
    inp_calc: I,
) -> Result<RebalQuote, QuoteRebalErr<I::Error>> {
    let RebalQuoteSolVal {
        sol_value,
        outstanding,
    } = quote_rebal_other_exact_out_sol_val(out, out_holding_balance, out_holding)
        .map_err(QuoteRebalErr::conv_infallible)?;
    let inp = *inp_calc
        .sol_to_lst(*sol_value.inp())
        .map_err(QuoteRebalErr::InpSvc)?
        .end();
    Ok(RebalQuote {
        qtys: RebalInpOut::const_from_destr(RebalQtysDestr {
            amt: InpOut::const_from_destr(InpOutDestr { inp, out }),
            sol_value,
        }),
        outstanding,
    })
}

#[inline]
pub const fn quote_rebal_other_exact_out_sol_val(
    out: u64,
    out_holding_balance: u64,
    out_holding: &HoldingV1LamportVals,
) -> Result<RebalQuoteSolVal, QuoteRebalErr<Infallible>> {
    if out > out_holding_balance || out_holding_balance == 0 {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(out, out_holding_balance),
        ));
    }
    let share = Ceil(Ratio {
        n: out,
        d: out_holding_balance,
    });
    let out_sol_val = match share.apply(*out_holding.sol_value()) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let outstanding = match share.apply(*out_holding.outstanding()) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    Ok(RebalQuoteSolVal {
        sol_value: InpOut::const_from_destr(InpOutDestr {
            inp: out_sol_val,
            out: out_sol_val,
        }),
        outstanding,
    })
}

#[cfg(kani)]
mod ver {
    use super::quote_rebal_other_exact_out_sol_val;
    use crate::typedefs::{HoldingV1LamportVals, HoldingV1LamportsDestr};

    /// Prove: sol_value.inp == sol_value.out (1:1 SOL value swap).
    #[kani::proof]
    #[kani::unwind(2)]
    fn other_sol_value_inp_equals_out() {
        let out: u64 = kani::any();
        let out_holding_balance: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();

        kani::assume(out > 0 && out <= 10_000_000);
        kani::assume(out_holding_balance > 0 && out_holding_balance <= 10_000_000);
        kani::assume(out <= out_holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10_000_000);
        kani::assume(sol_value > 0 && sol_value <= 10_000_000);

        let out_holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });

        let result = quote_rebal_other_exact_out_sol_val(out, out_holding_balance, &out_holding);
        if let Ok(q) = result {
            assert_eq!(
                *q.sol_value.inp(),
                *q.sol_value.out(),
                "sol_value.inp must equal sol_value.out"
            );
        }
    }

    /// Prove: outstanding from quote <= out_holding.outstanding.
    /// Ensures out_holding.outstanding won't underflow on subtraction.
    #[kani::proof]
    #[kani::unwind(2)]
    fn other_outstanding_bounded_by_holding() {
        let out: u64 = kani::any();
        let out_holding_balance: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();

        kani::assume(out > 0 && out <= 10);
        kani::assume(out_holding_balance > 0 && out_holding_balance <= 10);
        kani::assume(out <= out_holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10);
        kani::assume(sol_value > 0 && sol_value <= 10);

        let out_holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });

        let result = quote_rebal_other_exact_out_sol_val(out, out_holding_balance, &out_holding);
        if let Ok(q) = result {
            assert!(
                q.outstanding <= outstanding,
                "quote outstanding must not exceed holding outstanding"
            );
        }
    }

    /// Prove: sol_value from quote <= out_holding.sol_value.
    /// Ensures out_holding.sol_value won't underflow on subtraction.
    #[kani::proof]
    #[kani::unwind(2)]
    fn other_sol_value_bounded_by_holding() {
        let out: u64 = kani::any();
        let out_holding_balance: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();

        kani::assume(out > 0 && out <= 10);
        kani::assume(out_holding_balance > 0 && out_holding_balance <= 10);
        kani::assume(out <= out_holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10);
        kani::assume(sol_value > 0 && sol_value <= 10);

        let out_holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });

        let result = quote_rebal_other_exact_out_sol_val(out, out_holding_balance, &out_holding);
        if let Ok(q) = result {
            assert!(
                *q.sol_value.out() <= sol_value,
                "quote sol_value must not exceed holding sol_value"
            );
        }
    }

    /// Prove: full redemption (out == holding_balance) gives exact outstanding and sol_value.
    #[kani::proof]
    #[kani::unwind(2)]
    fn other_full_redemption_exact() {
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();
        let out_holding_balance: u64 = kani::any();

        kani::assume(out_holding_balance > 0 && out_holding_balance <= 10);
        kani::assume(outstanding > 0 && outstanding <= 10);
        kani::assume(sol_value > 0 && sol_value <= 10);

        let out_holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });

        let result = quote_rebal_other_exact_out_sol_val(
            out_holding_balance,
            out_holding_balance,
            &out_holding,
        );
        if let Ok(q) = result {
            assert_eq!(q.outstanding, outstanding, "full: outstanding must match");
            assert_eq!(*q.sol_value.out(), sol_value, "full: sol_value must match");
        }
    }

    /// Prove: subtracting quote values from holding doesn't underflow.
    #[kani::proof]
    #[kani::unwind(2)]
    fn other_subtraction_no_underflow() {
        let out: u64 = kani::any();
        let out_holding_balance: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();

        kani::assume(out > 0 && out <= 10);
        kani::assume(out_holding_balance > 0 && out_holding_balance <= 10);
        kani::assume(out <= out_holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10);
        kani::assume(sol_value > 0 && sol_value <= 10);

        let out_holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });

        let result = quote_rebal_other_exact_out_sol_val(out, out_holding_balance, &out_holding);
        if let Ok(q) = result {
            assert!(
                outstanding.checked_sub(q.outstanding).is_some(),
                "outstanding subtraction must not underflow"
            );
            assert!(
                sol_value.checked_sub(*q.sol_value.out()).is_some(),
                "sol_value subtraction must not underflow"
            );
        }
    }
}

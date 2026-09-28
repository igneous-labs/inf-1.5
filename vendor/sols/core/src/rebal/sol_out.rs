use core::convert::Infallible;

use inf1_svc_core::traits::SolValCalc;
use sanctum_u64_ratio::Floor;

use crate::{
    accounts::PoolLiqLamportVals,
    err::OutOfRangeErrU64,
    rebal::{
        verify_rebal_above_ceil, QuoteRebalErr, RRArgs, RebalInpOut, RebalQtysDestr, RebalQuote,
        RebalQuoteSolVal,
    },
    typedefs::Nanos,
    utils::{InpOut, InpOutDestr},
};

#[inline]
pub fn quote_rebal_sol_out_exact_out<I: SolValCalc>(
    out: u64,
    sol_out_loss_tol: Nanos,
    rr_args: &RRArgs,
    inp_calc: I,
) -> Result<RebalQuote, QuoteRebalErr<I::Error>> {
    let RebalQuoteSolVal {
        sol_value,
        outstanding,
    } = quote_rebal_sol_out_exact_out_sol_val(out, sol_out_loss_tol, rr_args)
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
pub const fn quote_rebal_sol_out_exact_out_sol_val(
    out: u64,
    sol_out_loss_tol: Nanos,
    rr: &RRArgs,
) -> Result<RebalQuoteSolVal, QuoteRebalErr<Infallible>> {
    let pool = rr.pool_lamports.liq_lamport_vals(rr.pool_acc_lamports);
    let q = match quote_rebal_sol_out_exact_out_sol_val_inner(out, &pool, sol_out_loss_tol) {
        Err(e) => return Err(e),
        Ok(x) => x,
    };
    match verify_rebal_above_ceil(rr, *q.sol_value.out()) {
        Ok(()) => Ok(q),
        Err(e) => Err(e),
    }
}

// quoting logic without checks for falling below ceiling
#[inline]
const fn quote_rebal_sol_out_exact_out_sol_val_inner(
    out: u64,
    pool: &PoolLiqLamportVals,
    sol_out_loss_tol: Nanos,
) -> Result<RebalQuoteSolVal, QuoteRebalErr<Infallible>> {
    let available = match pool.liq_avail_checked() {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    if out > available {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(out, available),
        ));
    }
    let max_sol_loss = match Floor(sol_out_loss_tol.into_ratio()).apply(out) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let inp_sol_val = match out.checked_sub(max_sol_loss) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    Ok(RebalQuoteSolVal {
        sol_value: InpOut::const_from_destr(InpOutDestr {
            inp: inp_sol_val,
            out,
        }),
        outstanding: out,
    })
}

#[cfg(kani)]
mod ver {
    use super::*;
    use crate::accounts::{PoolLiqLamportsDestr, PoolV1LamportVals, PoolV1LamportsDestr};

    /// Prove: outstanding always equals out AND sol_value.out == out.
    /// Tests on the full outer function (includes verify_rebal_above_ceil).
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_out_outstanding_equals_out() {
        let out: u64 = kani::any();
        let sol_out_loss_tol_raw: u32 = kani::any();
        let rent_exempt: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let protocol_fee: u64 = kani::any();
        let pool_acc_lamports: u64 = kani::any();
        let rrr_ceil_raw: u32 = kani::any();

        kani::assume(out > 0 && out <= 10_000_000);
        kani::assume(rent_exempt <= pool_acc_lamports);
        kani::assume(pool_acc_lamports <= 10_000_000);
        kani::assume(outstanding <= 10_000_000);
        kani::assume(protocol_fee <= 10_000_000);
        kani::assume(sol_out_loss_tol_raw <= 10_000_000);
        kani::assume(rrr_ceil_raw <= 1_000_000_000);

        let sol_out_loss_tol = Nanos::new(sol_out_loss_tol_raw).unwrap();
        let rrr_limit = Nanos::new(rrr_ceil_raw).unwrap();
        let pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt,
            outstanding,
            protocol_fee,
        });
        let rr = RRArgs {
            rrr_limit,
            pool_lamports,
            pool_acc_lamports,
        };

        let result = quote_rebal_sol_out_exact_out_sol_val(out, sol_out_loss_tol, &rr);
        if let Ok(q) = result {
            assert_eq!(q.outstanding, out);
            assert_eq!(*q.sol_value.out(), out);
        }
    }

    /// Prove: inp_sol_val <= out (loss tolerance means pool accepts less LST value).
    /// Tests the inner function directly for tractability.
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_out_inp_bounded_by_out() {
        let out: u64 = kani::any();
        let pool_total: u64 = kani::any();
        let pool_rent_exempt: u64 = kani::any();
        let sol_out_loss_tol_raw: u32 = kani::any();

        kani::assume(out > 0 && out <= 10_000_000);
        kani::assume(pool_rent_exempt <= pool_total);
        kani::assume(pool_total <= 10_000_000);
        kani::assume(sol_out_loss_tol_raw <= 10_000_000);

        let pool = PoolLiqLamportVals::const_from_destr(PoolLiqLamportsDestr {
            total: pool_total,
            rent_exempt: pool_rent_exempt,
        });
        let sol_out_loss_tol = Nanos::new(sol_out_loss_tol_raw).unwrap();

        let result = quote_rebal_sol_out_exact_out_sol_val_inner(out, &pool, sol_out_loss_tol);
        if let Ok(q) = result {
            assert!(
                *q.sol_value.inp() <= *q.sol_value.out(),
                "inp sol value must not exceed out sol value"
            );
        }
    }

    /// Prove: dep_due is preserved after a sol_out rebalance.
    /// pool_acc_lamports -= out, pool.outstanding += out => dep_due unchanged.
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_out_preserves_dep_due() {
        let out: u64 = kani::any();
        let pool_total: u64 = kani::any();
        let pool_rent_exempt: u64 = kani::any();
        let sol_out_loss_tol_raw: u32 = kani::any();
        let pool_outstanding: u64 = kani::any();
        let pool_protocol_fee: u64 = kani::any();

        kani::assume(out > 0 && out <= 10_000_000);
        kani::assume(pool_rent_exempt <= pool_total);
        kani::assume(pool_total <= 10_000_000);
        kani::assume(pool_outstanding <= 10_000_000);
        kani::assume(pool_protocol_fee <= 10_000_000);
        kani::assume(sol_out_loss_tol_raw <= 10_000_000);

        let pool = PoolLiqLamportVals::const_from_destr(PoolLiqLamportsDestr {
            total: pool_total,
            rent_exempt: pool_rent_exempt,
        });
        let sol_out_loss_tol = Nanos::new(sol_out_loss_tol_raw).unwrap();

        let result = quote_rebal_sol_out_exact_out_sol_val_inner(out, &pool, sol_out_loss_tol);
        if let Ok(q) = result {
            let pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
                rent_exempt: pool_rent_exempt,
                outstanding: pool_outstanding,
                protocol_fee: pool_protocol_fee,
            });
            let dep_due = match pool_lamports.dep_due_checked(pool_total) {
                Some(x) => x,
                None => return,
            };

            let new_pool_acc = match pool_total.checked_sub(*q.sol_value.out()) {
                Some(x) => x,
                None => return,
            };
            let new_outstanding = match pool_outstanding.checked_add(q.outstanding) {
                Some(x) => x,
                None => return,
            };
            let new_pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
                rent_exempt: pool_rent_exempt,
                outstanding: new_outstanding,
                protocol_fee: pool_protocol_fee,
            });
            let new_dep_due = match new_pool_lamports.dep_due_checked(new_pool_acc) {
                Some(x) => x,
                None => return,
            };

            assert_eq!(new_dep_due, dep_due, "dep_due must be preserved");
        }
    }
}

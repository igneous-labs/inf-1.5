use core::convert::Infallible;

use sanctum_u64_ratio::{Ceil, Floor, Ratio};

use crate::{
    accounts::{ProtocolV1FeeLamports, ProtocolV1FeeRatios},
    err::OutOfRangeErrU64,
    rebal::{QuoteRebalErr, RRArgs, RebalInpOut, RebalQtysDestr},
    typedefs::HoldingV1LamportVals,
    utils::{calc_protocol_fees, InpOut, InpOutDestr},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RebalSolInpPerm<Authed, Permless> {
    Authed(Authed),
    Permless(Permless),
}

pub type RebalSolInpRRArgs = RebalSolInpPerm<(), RRArgs>;

impl<Permless> Default for RebalSolInpPerm<(), Permless> {
    #[inline]
    fn default() -> Self {
        Self::Authed(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RebalSolInpQuote {
    pub qtys: RebalInpOut,

    /// Decrement in outstanding when rebalance is complete
    pub outstanding: u64,
    pub protocol_fees: ProtocolV1FeeLamports,
}

#[inline]
pub const fn quote_rebal_sol_inp_exact_out(
    out: u64,
    holding: &HoldingV1LamportVals,
    holding_balance: u64,
    protocol_fees: &ProtocolV1FeeRatios,
    rr_args: &RebalSolInpRRArgs,
) -> Result<RebalSolInpQuote, QuoteRebalErr<Infallible>> {
    let q = match quote_rebal_sol_inp_exact_out_inner(out, holding, holding_balance, protocol_fees)
    {
        Err(e) => return Err(e),
        Ok(x) => x,
    };
    if let RebalSolInpRRArgs::Permless(RRArgs {
        rrr_limit, // rrr_floor
        pool_lamports,
        pool_acc_lamports,
    }) = rr_args
    {
        let pf = match q.protocol_fees.total_checked() {
            None => return Err(QuoteRebalErr::Math),
            Some(x) => x,
        };
        // SOL reserves should not exceed floor
        // at end of rebal if permissionless, assuming exact rebalance.
        //
        // If not exact, user is free to donate extra SOL to the pool and
        // move reserve ratio beyond floor if he wants to for some reason
        let dep_due = match pool_lamports.dep_due_checked(*pool_acc_lamports) {
            None => return Err(QuoteRebalErr::Math),
            Some(x) => x,
        };
        let liq_avail = match pool_lamports.liq_avail_checked(*pool_acc_lamports) {
            None => return Err(QuoteRebalErr::Math),
            Some(x) => x,
        };
        let max_liq_avail = match Floor(rrr_limit.into_ratio()).apply(dep_due) {
            None => return Err(QuoteRebalErr::Math),
            Some(x) => x,
        };
        let max_liq_avail_inc = max_liq_avail.saturating_sub(liq_avail);
        let liq_avail_inc = match q.qtys.sol_value().inp().checked_sub(pf) {
            None => return Err(QuoteRebalErr::Math),
            Some(x) => x,
        };
        if liq_avail_inc > max_liq_avail_inc {
            return Err(QuoteRebalErr::RebalExceedLimit(
                OutOfRangeErrU64::actual_exceed_max(liq_avail_inc, max_liq_avail_inc),
            ));
        }
    }
    Ok(q)
}

// quoting logic without checks for permissionless case
#[inline]
const fn quote_rebal_sol_inp_exact_out_inner(
    out: u64,
    holding: &HoldingV1LamportVals,
    holding_balance: u64,
    protocol_fees: &ProtocolV1FeeRatios,
) -> Result<RebalSolInpQuote, QuoteRebalErr<Infallible>> {
    if out > holding_balance || holding_balance == 0 {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(out, holding_balance),
        ));
    }
    // Without this explicit check, `lamports_returning` and `inp` will be 0
    // below, allowing stealing of holding balance
    if *holding.outstanding() == 0 {
        return Err(QuoteRebalErr::RebalExceedLimit(
            OutOfRangeErrU64::actual_exceed_max(
                // any nonzero amt > 0
                1, 0,
            ),
        ));
    }
    let share = Ceil(Ratio {
        n: out,
        d: holding_balance,
    });
    let lamports_returning = match share.apply(*holding.outstanding()) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let protocol_fees = match calc_protocol_fees(lamports_returning, holding, protocol_fees) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let protocol_fee_lamports = match protocol_fees.total_checked() {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let out_sol_value = match share.apply(*holding.sol_value()) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let inp = match lamports_returning.checked_add(protocol_fee_lamports) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    Ok(RebalSolInpQuote {
        qtys: RebalInpOut::const_from_destr(RebalQtysDestr {
            amt: InpOut::const_from_destr(InpOutDestr { inp, out }),
            sol_value: InpOut::const_from_destr(InpOutDestr {
                inp,
                out: out_sol_value,
            }),
        }),
        outstanding: lamports_returning,
        protocol_fees,
    })
}

#[cfg(kani)]
mod ver {
    use super::quote_rebal_sol_inp_exact_out_inner;
    use crate::{
        accounts::{
            PoolV1LamportVals, PoolV1LamportsDestr, ProtocolV1FeeRatios, ProtocolV1FeesDestr,
        },
        typedefs::{HoldingV1LamportVals, HoldingV1LamportsDestr, Nanos},
        utils::protocol_fee_ratio,
    };

    /// Prove: inp == outstanding (lamports_returning) + protocol_fees.
    /// This is critical: the SOL entering the pool must exactly equal
    /// the outstanding being settled plus the protocol's cut.
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_inp_inp_is_returning_plus_fees() {
        let out: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();
        let holding_balance: u64 = kani::any();
        let fixed_fee_nanos: u32 = kani::any();
        let perf_fee_nanos: u32 = kani::any();

        kani::assume(out > 0 && out <= 10_000_000);
        kani::assume(holding_balance > 0 && holding_balance <= 10_000_000);
        kani::assume(out <= holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10_000_000);
        kani::assume(sol_value > 0 && sol_value <= 10_000_000);
        kani::assume(fixed_fee_nanos < 1_000_000_000);
        kani::assume(perf_fee_nanos < 1_000_000_000);

        let holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });
        let pf = ProtocolV1FeeRatios::const_from_destr(ProtocolV1FeesDestr {
            fixed: protocol_fee_ratio(Nanos::new(fixed_fee_nanos).unwrap()),
            perf: protocol_fee_ratio(Nanos::new(perf_fee_nanos).unwrap()),
        });

        let result = quote_rebal_sol_inp_exact_out_inner(out, &holding, holding_balance, &pf);

        if let Ok(q) = result {
            let total_pf = q.protocol_fees.total_checked().unwrap();
            assert_eq!(
                *q.qtys.amt().inp(),
                q.outstanding + total_pf,
                "inp must equal lamports_returning + protocol_fees"
            );
        }
    }

    /// Prove: lamports_returning (outstanding) is bounded by holding.outstanding.
    /// Since out <= holding_balance, ceil(out/holding_balance * outstanding) <= outstanding.
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_inp_outstanding_bounded() {
        let out: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();
        let holding_balance: u64 = kani::any();
        let fixed_fee_nanos: u32 = kani::any();
        let perf_fee_nanos: u32 = kani::any();

        kani::assume(out > 0 && out <= 10);
        kani::assume(holding_balance > 0 && holding_balance <= 10);
        kani::assume(out <= holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10);
        kani::assume(sol_value > 0 && sol_value <= 10);
        kani::assume(fixed_fee_nanos < 1_000_000_000);
        kani::assume(perf_fee_nanos < 1_000_000_000);

        let holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });
        let pf = ProtocolV1FeeRatios::const_from_destr(ProtocolV1FeesDestr {
            fixed: protocol_fee_ratio(Nanos::new(fixed_fee_nanos).unwrap()),
            perf: protocol_fee_ratio(Nanos::new(perf_fee_nanos).unwrap()),
        });

        let result = quote_rebal_sol_inp_exact_out_inner(out, &holding, holding_balance, &pf);

        if let Ok(q) = result {
            assert!(
                q.outstanding <= outstanding,
                "lamports_returning must not exceed holding outstanding"
            );
        }
    }

    /// Prove: dep_due is preserved after sol_inp rebalance.
    /// pool_acc_lamports += inp, pool.outstanding -= returning, pool.protocol_fee += pf
    /// => dep_due unchanged.
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_inp_preserves_dep_due() {
        let out: u64 = kani::any();
        let h_outstanding: u64 = kani::any();
        let h_sol_value: u64 = kani::any();
        let holding_balance: u64 = kani::any();
        let fixed_fee_nanos: u32 = kani::any();
        let perf_fee_nanos: u32 = kani::any();

        kani::assume(out > 0 && out <= 10);
        kani::assume(holding_balance > 0 && holding_balance <= 10);
        kani::assume(out <= holding_balance);
        kani::assume(h_outstanding > 0 && h_outstanding <= 10);
        kani::assume(h_sol_value > 0 && h_sol_value <= 10);
        kani::assume(fixed_fee_nanos < 1_000_000_000);
        kani::assume(perf_fee_nanos < 1_000_000_000);

        let holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding: h_outstanding,
            sol_value: h_sol_value,
        });
        let pf_ratios = ProtocolV1FeeRatios::const_from_destr(ProtocolV1FeesDestr {
            fixed: protocol_fee_ratio(Nanos::new(fixed_fee_nanos).unwrap()),
            perf: protocol_fee_ratio(Nanos::new(perf_fee_nanos).unwrap()),
        });

        let rent_exempt: u64 = kani::any();
        let pool_outstanding: u64 = kani::any();
        let pool_pf: u64 = kani::any();
        let pool_acc: u64 = kani::any();

        kani::assume(rent_exempt <= pool_acc);
        kani::assume(pool_acc <= 100);
        kani::assume(pool_outstanding <= 100);
        kani::assume(pool_pf <= 100);

        let pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt,
            outstanding: pool_outstanding,
            protocol_fee: pool_pf,
        });
        let dep_due = match pool_lamports.dep_due_checked(pool_acc) {
            Some(x) => x,
            None => return,
        };

        let result =
            quote_rebal_sol_inp_exact_out_inner(out, &holding, holding_balance, &pf_ratios);
        if let Ok(q) = result {
            let total_pf = match q.protocol_fees.total_checked() {
                Some(x) => x,
                None => return,
            };

            let new_pool_acc = match pool_acc.checked_add(*q.qtys.amt().inp()) {
                Some(x) => x,
                None => return,
            };
            let new_outstanding = match pool_outstanding.checked_sub(q.outstanding) {
                Some(x) => x,
                None => return,
            };
            let new_pf = match pool_pf.checked_add(total_pf) {
                Some(x) => x,
                None => return,
            };

            let new_pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
                rent_exempt,
                outstanding: new_outstanding,
                protocol_fee: new_pf,
            });
            let new_dep_due = match new_pool_lamports.dep_due_checked(new_pool_acc) {
                Some(x) => x,
                None => return,
            };

            assert_eq!(new_dep_due, dep_due, "dep_due must be preserved");
        }
    }

    /// Prove: when sol_value <= outstanding (no gains / depreciation),
    /// performance fee must be 0. Only fixed fee should apply.
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_inp_no_perf_fee_when_no_gains() {
        let out: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();
        let holding_balance: u64 = kani::any();
        let fixed_fee_nanos: u32 = kani::any();
        let perf_fee_nanos: u32 = kani::any();

        kani::assume(out > 0 && out <= 10_000_000);
        kani::assume(holding_balance > 0 && holding_balance <= 10_000_000);
        kani::assume(out <= holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10_000_000);
        kani::assume(sol_value > 0 && sol_value <= 10_000_000);
        kani::assume(sol_value <= outstanding);
        kani::assume(fixed_fee_nanos < 1_000_000_000);
        kani::assume(perf_fee_nanos < 1_000_000_000);

        let holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });
        let pf = ProtocolV1FeeRatios::const_from_destr(ProtocolV1FeesDestr {
            fixed: protocol_fee_ratio(Nanos::new(fixed_fee_nanos).unwrap()),
            perf: protocol_fee_ratio(Nanos::new(perf_fee_nanos).unwrap()),
        });

        let result = quote_rebal_sol_inp_exact_out_inner(out, &holding, holding_balance, &pf);

        if let Ok(q) = result {
            assert_eq!(
                *q.protocol_fees.perf(),
                0,
                "perf fee must be 0 when sol_value <= outstanding"
            );
        }
    }

    /// Prove: sol_value.inp equals amt.inp (for SOL input, both are the same).
    #[kani::proof]
    #[kani::unwind(2)]
    fn sol_inp_sol_value_inp_equals_amt_inp() {
        let out: u64 = kani::any();
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();
        let holding_balance: u64 = kani::any();
        let fixed_fee_nanos: u32 = kani::any();
        let perf_fee_nanos: u32 = kani::any();

        kani::assume(out > 0 && out <= 10_000_000);
        kani::assume(holding_balance > 0 && holding_balance <= 10_000_000);
        kani::assume(out <= holding_balance);
        kani::assume(outstanding > 0 && outstanding <= 10_000_000);
        kani::assume(sol_value > 0 && sol_value <= 10_000_000);
        kani::assume(fixed_fee_nanos < 1_000_000_000);
        kani::assume(perf_fee_nanos < 1_000_000_000);

        let holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });
        let pf = ProtocolV1FeeRatios::const_from_destr(ProtocolV1FeesDestr {
            fixed: protocol_fee_ratio(Nanos::new(fixed_fee_nanos).unwrap()),
            perf: protocol_fee_ratio(Nanos::new(perf_fee_nanos).unwrap()),
        });

        let result = quote_rebal_sol_inp_exact_out_inner(out, &holding, holding_balance, &pf);

        if let Ok(q) = result {
            assert_eq!(
                *q.qtys.sol_value().inp(),
                *q.qtys.amt().inp(),
                "sol_value.inp must equal amt.inp for SOL input"
            );
        }
    }
}

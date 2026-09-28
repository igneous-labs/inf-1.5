use core::convert::Infallible;

use sanctum_fee_ratio::Fee;
use sanctum_u64_ratio::{Ceil, Floor, Ratio};

use crate::{
    accounts::ProtocolV1FeeLamports,
    rebal::{
        quote_claim_holding, verify_rebal_above_ceil, ClaimHoldingQuote, QuoteClaimHoldingArgs,
        QuoteRebalErr, RRArgs, RebalQtys, RebalQtysDestr,
    },
    typedefs::{HoldingV1LamportVals, Nanos},
    utils::{InpOut, InpOutDestr},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SwapHoldingQuote {
    /// The input and output SOLS amounts that will leave and enter
    /// the user's wallet
    pub qtys: InpOut<u64>,

    /// If negative, then outstanding is incremented instead
    /// due to protocol fees > sol being claimed
    pub inp_outstanding_dec: i128,

    /// The intermediate holding used to swap between these 2 SOLS.
    ///
    /// SOL value qty is before out's deposit fee
    pub holding: RebalQtys<u64>,

    /// Deposit fee charged by out pool
    pub deposit_fee_lamports: u64,

    /// Protocol fees charged on the ClaimHolding leg of inp pool
    pub protocol_fees: ProtocolV1FeeLamports,
}

#[inline]
pub const fn quote_swap_holding(
    inp_amt: u64,
    inp: &QuoteClaimHoldingArgs,
    out: &QuoteDepositHoldingArgs,
) -> Result<SwapHoldingQuote, QuoteRebalErr<Infallible>> {
    let chq = match quote_claim_holding(inp_amt, inp) {
        Err(e) => return Err(e),
        Ok(x) => x,
    };
    swap_holding_quote_from_claim(&chq, out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuoteDepositHoldingArgs {
    /// The pool's entire portfolio (sum of all holdings)
    /// represented as a single holding
    pub portfolio: HoldingV1LamportVals,
    pub deposit_fee: Nanos,
    pub rr: RRArgs,
}

#[inline]
pub const fn swap_holding_quote_from_claim(
    ClaimHoldingQuote {
        qtys,
        outstanding_dec: inp_outstanding_dec,
        protocol_fees,
    }: &ClaimHoldingQuote,
    QuoteDepositHoldingArgs {
        portfolio,
        deposit_fee,
        rr,
    }: &QuoteDepositHoldingArgs,
) -> Result<SwapHoldingQuote, QuoteRebalErr<Infallible>> {
    type F = Fee<Ceil<Ratio<u32, u32>>>;

    let fee: Fee<Ceil<Ratio<u32, u32>>> = match F::new(deposit_fee.into_ratio()) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };

    let incoming_lamports = *qtys.sol_value().out();
    let incoming_lamports_aft_fee = match fee.apply(incoming_lamports) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let deposit_fee_lamports = incoming_lamports_aft_fee.fee();
    let incoming_lamports_user_due = incoming_lamports_aft_fee.rem();

    let sol_liq_avail = match rr.pool_lamports.liq_avail_checked(rr.pool_acc_lamports) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let pool_outstanding = match portfolio.outstanding().checked_add(sol_liq_avail) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let pool_total_sol_value = match portfolio.sol_value().checked_add(sol_liq_avail) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };

    // normalize to pool's existing `outstanding : sol value` to prevent stealing
    // of mismatched appreciation
    let out_outstanding_inc = if pool_outstanding == 0 || pool_total_sol_value == 0 {
        incoming_lamports_user_due
    } else {
        match Floor::new(Ratio {
            n: pool_outstanding,
            d: pool_total_sol_value,
        })
        .apply(incoming_lamports_user_due)
        {
            None => return Err(QuoteRebalErr::Math),
            Some(x) => x,
        }
    };

    // to check if we are within limits, call verify_rebal_above_ceil
    // as if we just received out_outstanding_inc in SOL deposits and
    // are rebalancing that out of SOL
    let sim_pool_acc_lamports = match rr.pool_acc_lamports.checked_add(out_outstanding_inc) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    match verify_rebal_above_ceil(
        &RRArgs {
            rrr_limit: rr.rrr_limit,
            pool_lamports: rr.pool_lamports,
            pool_acc_lamports: sim_pool_acc_lamports,
        },
        out_outstanding_inc,
    ) {
        Err(e) => Err(e),
        Ok(()) => Ok(SwapHoldingQuote {
            qtys: InpOut::const_from_destr(InpOutDestr {
                inp: *qtys.amt().inp(),
                out: out_outstanding_inc,
            }),
            inp_outstanding_dec: *inp_outstanding_dec,
            holding: RebalQtys::const_from_destr(RebalQtysDestr {
                amt: *qtys.amt().out(),
                sol_value: *qtys.sol_value().out(),
            }),
            deposit_fee_lamports,
            protocol_fees: *protocol_fees,
        }),
    }
}

#[cfg(kani)]
mod ver {
    use super::{swap_holding_quote_from_claim, QuoteDepositHoldingArgs, QuoteRebalErr, RRArgs};
    use crate::accounts::spec::is_pool_solvent;
    use crate::accounts::{PoolLiqLamportVals, PoolLiqLamportsDestr, ProtocolV1FeeRatios};
    use crate::rebal::{quote_claim_holding, QuoteClaimHoldingArgs};
    use crate::utils::protocol_fee_ratio;
    use crate::{
        accounts::{
            PoolV1LamportVals, PoolV1LamportsDestr, ProtocolV1FeeLamports, ProtocolV1FeesDestr,
        },
        rebal::{ClaimHoldingQuote, RebalInpOut, RebalQtysDestr},
        typedefs::{HoldingV1LamportVals, HoldingV1LamportsDestr, Nanos},
        utils::{InpOut, InpOutDestr},
    };

    /// Prove: deposit fee ≤ sol_value.out from claim.
    /// Deposit fee splits sol_value into fee + user_share before pool-ratio normalization.
    #[kani::proof]
    #[kani::unwind(2)]
    fn swap_holding_fee_decomposition() {
        let claim_inp: u64 = kani::any();
        let claim_out: u64 = kani::any();
        let sol_value_inp: u64 = kani::any();
        let sol_value_out: u64 = kani::any();
        let outstanding_dec: i128 = kani::any();
        let pf_fixed: u64 = kani::any();
        let pf_perf: u64 = kani::any();
        let deposit_fee_nanos: u32 = kani::any();

        // tight bounds needed: Ratio math uses u128 math which creates huge SAT formulas
        const TIGHT_LAMPORT_BOUND_CLAIM: u64 = 10;
        kani::assume(claim_inp > 0 && claim_inp <= TIGHT_LAMPORT_BOUND_CLAIM);
        kani::assume(claim_out <= TIGHT_LAMPORT_BOUND_CLAIM);
        kani::assume(sol_value_inp > 0 && sol_value_inp <= TIGHT_LAMPORT_BOUND_CLAIM);
        kani::assume(sol_value_out > 0 && sol_value_out <= TIGHT_LAMPORT_BOUND_CLAIM);
        kani::assume(
            outstanding_dec >= -i128::from(TIGHT_LAMPORT_BOUND_CLAIM)
                && outstanding_dec <= TIGHT_LAMPORT_BOUND_CLAIM.into(),
        );
        kani::assume(pf_fixed <= TIGHT_LAMPORT_BOUND_CLAIM);
        kani::assume(pf_perf <= TIGHT_LAMPORT_BOUND_CLAIM);
        kani::assume(deposit_fee_nanos <= 1_000_000_000);

        let chq = ClaimHoldingQuote {
            qtys: RebalInpOut::const_from_destr(RebalQtysDestr {
                amt: InpOut::const_from_destr(InpOutDestr {
                    inp: claim_inp,
                    out: claim_out,
                }),
                sol_value: InpOut::const_from_destr(InpOutDestr {
                    inp: sol_value_inp,
                    out: sol_value_out,
                }),
            }),
            outstanding_dec,
            protocol_fees: ProtocolV1FeeLamports::const_from_destr(ProtocolV1FeesDestr {
                fixed: pf_fixed,
                perf: pf_perf,
            }),
        };

        let rent_exempt: u64 = kani::any();
        let pool_outstanding: u64 = kani::any();
        let portfolio_sol_value: u64 = kani::any();
        let pool_pf: u64 = kani::any();
        let pool_acc: u64 = kani::any();
        let rrr_nanos: u32 = kani::any();

        const TIGHT_LAMPORT_BOUND_OUT_POOL: u64 = 100;
        kani::assume(rent_exempt <= pool_acc);
        kani::assume(pool_acc <= TIGHT_LAMPORT_BOUND_OUT_POOL);
        kani::assume(pool_outstanding <= TIGHT_LAMPORT_BOUND_OUT_POOL);
        kani::assume(portfolio_sol_value <= TIGHT_LAMPORT_BOUND_OUT_POOL);
        kani::assume(pool_pf <= TIGHT_LAMPORT_BOUND_OUT_POOL);
        kani::assume(rrr_nanos <= 1_000_000_000);

        let rr = RRArgs {
            rrr_limit: Nanos::new(rrr_nanos).unwrap(),
            pool_lamports: PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
                rent_exempt,
                outstanding: pool_outstanding,
                protocol_fee: pool_pf,
            }),
            pool_acc_lamports: pool_acc,
        };

        let deposit_fee_struct = Nanos::new(deposit_fee_nanos).unwrap();
        let portfolio = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding: pool_outstanding,
            sol_value: portfolio_sol_value,
        });
        let args = QuoteDepositHoldingArgs {
            portfolio,
            deposit_fee: deposit_fee_struct,
            rr,
        };
        let result = swap_holding_quote_from_claim(&chq, &args);

        if let Ok(q) = result {
            assert!(
                q.deposit_fee_lamports <= sol_value_out,
                "deposit fee must not exceed sol_value.out"
            );
        }
    }

    /// Prove no value extraction: SOLS tokens minted × pool sol_value
    /// ≤ contributed SOL × pool outstanding.
    /// I.e. user cannot extract more pool value than they contributed,
    /// after adjusting for pool's overall ratio.
    #[kani::proof]
    #[kani::unwind(2)]
    fn swap_holding_no_value_extraction() {
        let sol_value_out: u64 = kani::any();
        let deposit_fee_nanos: u32 = kani::any();
        let rent_exempt: u64 = kani::any();
        let pool_outstanding: u64 = kani::any();
        let pool_pf: u64 = kani::any();
        let pool_acc: u64 = kani::any();
        let rrr_nanos: u32 = kani::any();
        let portfolio_outstanding: u64 = kani::any();
        let portfolio_sol_value: u64 = kani::any();

        const TIGHT: u64 = 10;
        kani::assume(sol_value_out > 0 && sol_value_out <= TIGHT);
        kani::assume(deposit_fee_nanos <= 1_000_000_000);
        kani::assume(rent_exempt <= pool_acc);
        kani::assume(pool_acc <= TIGHT);
        kani::assume(pool_outstanding <= TIGHT);
        kani::assume(pool_pf <= TIGHT);
        let pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt,
            outstanding: pool_outstanding,
            protocol_fee: pool_pf,
        });
        kani::assume(is_pool_solvent((pool_lamports, pool_acc)));
        kani::assume(rrr_nanos <= 1_000_000_000);
        kani::assume(portfolio_outstanding <= TIGHT);
        kani::assume(portfolio_sol_value <= TIGHT);

        let chq = ClaimHoldingQuote {
            qtys: RebalInpOut::const_from_destr(RebalQtysDestr {
                amt: InpOut::const_from_destr(InpOutDestr { inp: 5, out: 5 }),
                sol_value: InpOut::const_from_destr(InpOutDestr {
                    inp: 5,
                    out: sol_value_out,
                }),
            }),
            outstanding_dec: 5,
            protocol_fees: ProtocolV1FeeLamports::const_from_destr(ProtocolV1FeesDestr {
                fixed: 0,
                perf: 0,
            }),
        };

        let deposit_fee = Nanos::new(deposit_fee_nanos).unwrap();

        let sol_liq_avail = pool_acc - rent_exempt;
        let total_pool_outstanding = portfolio_outstanding + sol_liq_avail;
        let total_pool_sol_value = portfolio_sol_value + sol_liq_avail;

        let rr = RRArgs {
            rrr_limit: Nanos::new(rrr_nanos).unwrap(),
            pool_lamports,
            pool_acc_lamports: pool_acc,
        };

        let portfolio = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding: portfolio_outstanding,
            sol_value: portfolio_sol_value,
        });
        let args = QuoteDepositHoldingArgs {
            portfolio,
            deposit_fee,
            rr,
        };
        let result = swap_holding_quote_from_claim(&chq, &args);

        match result {
            Ok(q) => {
                let contributed_sol_value = sol_value_out - q.deposit_fee_lamports;
                let out = *q.qtys.out();
                // In degenerate case (no pool ratio), 1:1 minting:
                // user gets exactly their contributed SOL value as SOLS tokens.
                if total_pool_outstanding == 0 || total_pool_sol_value == 0 {
                    assert_eq!(
                        out, contributed_sol_value,
                        "degenerate path: 1:1 minting, out must equal contributed sol value"
                    );
                } else {
                    assert!(
                        (out as u128) * (total_pool_sol_value as u128)
                            <= (contributed_sol_value as u128) * (total_pool_outstanding as u128),
                        "no value extraction: out*sol_value <= contributed_sol*outstanding"
                    );
                }
            }
            Err(QuoteRebalErr::RebalExceedLimit(_)) => {
                // RRR ceiling hit, legitimate business path
            }
            Err(QuoteRebalErr::Math | QuoteRebalErr::NotEnoughLiquidity(_)) => {
                // Unreachable with tight bounds; prove it
                panic!();
            }
        }
    }

    /// Prove: the input token amount is preserved from the claim quote.
    /// This is a pass-through — normalization does not affect qtys.inp.
    #[kani::proof]
    #[kani::unwind(2)]
    fn swap_holding_input_preserved() {
        let claim_inp: u64 = kani::any();
        let sol_value_out: u64 = kani::any();
        let deposit_fee_nanos: u32 = kani::any();
        let rent_exempt: u64 = kani::any();
        let pool_outstanding: u64 = kani::any();
        let pool_pf: u64 = kani::any();
        let pool_acc: u64 = kani::any();
        let rrr_nanos: u32 = kani::any();
        let portfolio_outstanding: u64 = kani::any();
        let portfolio_sol_value: u64 = kani::any();

        const TIGHT: u64 = 10;
        kani::assume(claim_inp > 0 && claim_inp <= TIGHT);
        kani::assume(sol_value_out > 0 && sol_value_out <= TIGHT);
        kani::assume(deposit_fee_nanos <= 1_000_000_000);
        kani::assume(rent_exempt <= pool_acc);
        kani::assume(pool_acc <= TIGHT);
        kani::assume(pool_outstanding <= TIGHT);
        kani::assume(pool_pf <= TIGHT);
        let pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt,
            outstanding: pool_outstanding,
            protocol_fee: pool_pf,
        });
        kani::assume(is_pool_solvent((pool_lamports, pool_acc)));
        kani::assume(rrr_nanos <= 1_000_000_000);
        kani::assume(portfolio_outstanding <= TIGHT);
        kani::assume(portfolio_sol_value <= TIGHT);

        let chq = ClaimHoldingQuote {
            qtys: RebalInpOut::const_from_destr(RebalQtysDestr {
                amt: InpOut::const_from_destr(InpOutDestr {
                    inp: claim_inp,
                    out: 5,
                }),
                sol_value: InpOut::const_from_destr(InpOutDestr {
                    inp: claim_inp,
                    out: sol_value_out,
                }),
            }),
            outstanding_dec: 5,
            protocol_fees: ProtocolV1FeeLamports::const_from_destr(ProtocolV1FeesDestr {
                fixed: 0,
                perf: 0,
            }),
        };

        let deposit_fee = Nanos::new(deposit_fee_nanos).unwrap();

        let rr = RRArgs {
            rrr_limit: Nanos::new(rrr_nanos).unwrap(),
            pool_lamports,
            pool_acc_lamports: pool_acc,
        };

        let portfolio = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding: portfolio_outstanding,
            sol_value: portfolio_sol_value,
        });
        let args = QuoteDepositHoldingArgs {
            portfolio,
            deposit_fee,
            rr,
        };
        let result = swap_holding_quote_from_claim(&chq, &args);

        match result {
            Ok(q) => {
                assert_eq!(
                    *q.qtys.inp(),
                    claim_inp,
                    "input amount must be preserved from claim"
                );
            }
            Err(QuoteRebalErr::RebalExceedLimit(_)) => {
                // RRR ceiling hit, legitimate business path
            }
            Err(QuoteRebalErr::Math | QuoteRebalErr::NotEnoughLiquidity(_)) => {
                panic!("Math/NotEnoughLiquidity unreachable with tight bounds");
            }
        }
    }

    /// Prove: holding sol_value and amt are preserved from the claim quote.
    /// These fields are passed through verbatim — normalization does not modify them.
    #[kani::proof]
    #[kani::unwind(2)]
    fn swap_holding_holding_fields_preserved() {
        let claim_out: u64 = kani::any();
        let sol_value_out: u64 = kani::any();
        let deposit_fee_nanos: u32 = kani::any();
        let rent_exempt: u64 = kani::any();
        let pool_outstanding: u64 = kani::any();
        let pool_pf: u64 = kani::any();
        let pool_acc: u64 = kani::any();
        let rrr_nanos: u32 = kani::any();
        let portfolio_outstanding: u64 = kani::any();
        let portfolio_sol_value: u64 = kani::any();

        const TIGHT: u64 = 10;
        kani::assume(claim_out > 0 && claim_out <= TIGHT);
        kani::assume(sol_value_out > 0 && sol_value_out <= TIGHT);
        kani::assume(deposit_fee_nanos <= 1_000_000_000);
        kani::assume(rent_exempt <= pool_acc);
        kani::assume(pool_acc <= TIGHT);
        kani::assume(pool_outstanding <= TIGHT);
        kani::assume(pool_pf <= TIGHT);
        let pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt,
            outstanding: pool_outstanding,
            protocol_fee: pool_pf,
        });
        kani::assume(is_pool_solvent((pool_lamports, pool_acc)));
        kani::assume(rrr_nanos <= 1_000_000_000);
        kani::assume(portfolio_outstanding <= TIGHT);
        kani::assume(portfolio_sol_value <= TIGHT);

        let chq = ClaimHoldingQuote {
            qtys: RebalInpOut::const_from_destr(RebalQtysDestr {
                amt: InpOut::const_from_destr(InpOutDestr {
                    inp: 5,
                    out: claim_out,
                }),
                sol_value: InpOut::const_from_destr(InpOutDestr {
                    inp: 5,
                    out: sol_value_out,
                }),
            }),
            outstanding_dec: 5,
            protocol_fees: ProtocolV1FeeLamports::const_from_destr(ProtocolV1FeesDestr {
                fixed: 0,
                perf: 0,
            }),
        };

        let deposit_fee = Nanos::new(deposit_fee_nanos).unwrap();

        let rr = RRArgs {
            rrr_limit: Nanos::new(rrr_nanos).unwrap(),
            pool_lamports,
            pool_acc_lamports: pool_acc,
        };

        let portfolio = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding: portfolio_outstanding,
            sol_value: portfolio_sol_value,
        });
        let args = QuoteDepositHoldingArgs {
            portfolio,
            deposit_fee,
            rr,
        };
        let result = swap_holding_quote_from_claim(&chq, &args);

        match result {
            Ok(q) => {
                assert_eq!(
                    *q.holding.amt(),
                    claim_out,
                    "holding.amt must be claim's amt.out"
                );
                assert_eq!(
                    *q.holding.sol_value(),
                    sol_value_out,
                    "holding.sol_value must be claim's sol_value.out"
                );
            }
            Err(QuoteRebalErr::RebalExceedLimit(_)) => {
                // RRR ceiling hit, legitimate business path
            }
            Err(QuoteRebalErr::Math | QuoteRebalErr::NotEnoughLiquidity(_)) => {
                panic!("Math/NotEnoughLiquidity unreachable with tight bounds");
            }
        }
    }

    /// End-to-end swap_other: run quote_claim_holding -> swap_holding_quote_from_claim,
    /// then verify both inp and out pool surplus is preserved.
    ///
    /// Surplus preservation holds regardless of pool ratio because both
    /// dep_due and mint_supply increase by the same delta (SOLS tokens minted).
    #[kani::proof]
    #[kani::unwind(2)]
    fn swap_other_end_to_end_surplus() {
        // --- inp pool (claim_holding) params ---
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();
        let holding_balance: u64 = kani::any();
        let claim_lamports: u64 = kani::any();
        let pool_liq_rent_exempt: u64 = kani::any();
        let inp_fixed_fee: u32 = kani::any();
        let inp_perf_fee: u32 = kani::any();

        const TIGHT: u64 = 10;
        kani::assume(outstanding > 0 && outstanding <= TIGHT);
        kani::assume(sol_value > 0 && sol_value <= TIGHT);
        kani::assume(holding_balance > 0 && holding_balance <= TIGHT);
        kani::assume(claim_lamports > 0 && claim_lamports <= outstanding);
        kani::assume(pool_liq_rent_exempt <= TIGHT);
        kani::assume(inp_fixed_fee < 1_000_000_000);
        kani::assume(inp_perf_fee < 1_000_000_000);

        let holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });
        // quote_claim_holding requires pool has no liquid SOL (liq_avail == 0)
        let pool_liq = PoolLiqLamportVals::const_from_destr(PoolLiqLamportsDestr {
            total: pool_liq_rent_exempt,
            rent_exempt: pool_liq_rent_exempt,
        });
        let protocol_fees = ProtocolV1FeeRatios::const_from_destr(ProtocolV1FeesDestr {
            fixed: protocol_fee_ratio(Nanos::new(inp_fixed_fee).unwrap()),
            perf: protocol_fee_ratio(Nanos::new(inp_perf_fee).unwrap()),
        });

        let portfolio_holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });
        let inp_args = QuoteClaimHoldingArgs {
            portfolio: portfolio_holding,
            holding,
            holding_balance,
            protocol_fees,
            pool: pool_liq,
        };

        let chq = match quote_claim_holding(claim_lamports, &inp_args) {
            Ok(x) => x,
            Err(_) => return,
        };

        // --- out pool (swap_holding) params ---
        let out_rent: u64 = kani::any();
        let out_outstanding: u64 = kani::any();
        let out_pf: u64 = kani::any();
        let out_acc: u64 = kani::any();
        let out_mint_supply: u64 = kani::any();
        let out_portfolio_outstanding: u64 = kani::any();
        let out_portfolio_sol_value: u64 = kani::any();
        let deposit_fee_nanos: u32 = kani::any();
        let rrr_nanos: u32 = kani::any();

        kani::assume(out_rent > 0 && out_rent <= TIGHT);
        kani::assume(out_acc >= out_rent && out_acc <= TIGHT);
        kani::assume(out_outstanding <= TIGHT);
        kani::assume(out_pf <= TIGHT);
        kani::assume(out_portfolio_outstanding <= TIGHT);
        kani::assume(out_portfolio_sol_value <= TIGHT);
        kani::assume(deposit_fee_nanos <= 1_000_000_000);
        kani::assume(rrr_nanos <= 1_000_000_000);

        let out_pool = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: out_rent,
            outstanding: out_outstanding,
            protocol_fee: out_pf,
        });
        kani::assume(is_pool_solvent((out_pool, out_acc)));

        let out_dep_due_before = match out_pool.dep_due_checked(out_acc) {
            Some(x) => x,
            None => return,
        };
        kani::assume(out_mint_supply <= out_dep_due_before);

        let deposit_fee = Nanos::new(deposit_fee_nanos).unwrap();

        let out_portfolio = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding: out_portfolio_outstanding,
            sol_value: out_portfolio_sol_value,
        });
        let shargs = QuoteDepositHoldingArgs {
            portfolio: out_portfolio,
            deposit_fee,
            rr: RRArgs {
                rrr_limit: Nanos::new(rrr_nanos).unwrap(),
                pool_lamports: out_pool,
                pool_acc_lamports: out_acc,
            },
        };

        let shq = match swap_holding_quote_from_claim(&chq, &shargs) {
            Ok(x) => x,
            Err(QuoteRebalErr::RebalExceedLimit(_)) => return,
            Err(QuoteRebalErr::Math | QuoteRebalErr::NotEnoughLiquidity(_)) => {
                panic!("Math/NotEnoughLiquidity unreachable with tight bounds");
            }
        };

        let delta = *shq.qtys.out();

        let new_outstanding = out_outstanding + delta;
        let new_out_pool = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: out_rent,
            outstanding: new_outstanding,
            protocol_fee: out_pf,
        });
        let new_dep_due = match new_out_pool.dep_due_checked(out_acc) {
            Some(x) => x,
            None => return,
        };
        let new_mint_supply = out_mint_supply + delta;

        assert_eq!(
            new_dep_due - new_mint_supply,
            out_dep_due_before - out_mint_supply,
            "out pool surplus must be preserved after swap_other"
        );

        let pf_total = match chq.protocol_fees.total_checked() {
            Some(x) => x,
            None => return,
        };
        let expected_outstanding_dec = (claim_lamports as i128) - (pf_total as i128);
        assert_eq!(
            chq.outstanding_dec, expected_outstanding_dec,
            "inp pool: outstanding_dec must equal claim_lamports - pf_total"
        );
    }
}

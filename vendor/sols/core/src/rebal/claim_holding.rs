use core::convert::Infallible;

use sanctum_u64_ratio::{Floor, Ratio};

use crate::{
    accounts::{PoolLiqLamportVals, ProtocolV1FeeLamports, ProtocolV1FeeRatios},
    err::OutOfRangeErrU64,
    rebal::{QuoteRebalErr, RebalInpOut, RebalQtysDestr},
    typedefs::HoldingV1LamportVals,
    utils::{calc_protocol_fees, InpOut, InpOutDestr},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuoteClaimHoldingArgs {
    /// The pool's entire portfolio (sum of all holdings)
    /// represented as a single holding
    pub portfolio: HoldingV1LamportVals,
    pub holding: HoldingV1LamportVals,
    pub holding_balance: u64,
    pub protocol_fees: ProtocolV1FeeRatios,
    pub pool: PoolLiqLamportVals,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClaimHoldingQuote {
    /// inp qty = inp SOL value = SOLS token being redeemed
    ///
    /// out = holding token that is being claimed
    pub qtys: RebalInpOut,

    /// If negative, then outstanding is incremented instead
    /// due to protocol fees > sol being claimed
    pub outstanding_dec: i128,

    /// In terms of lamports (SOL)
    pub protocol_fees: ProtocolV1FeeLamports,
}

/// # Note
/// - Does not check that `claim_lamports <= pool_surplus`.
///   See [`crate::utils::vercomp_mint_claim_amt`] for that
/// - Returns [`QuoteRebalErr::NotEnoughLiquidity`] if portfolio does not have
///   enough SOL value to cover protocol fees. This means pools can be bricked by extreme
///   loss of SOL value events. Require external donors to make pool solvent again in those cases.
#[inline]
pub const fn quote_claim_holding(
    claim_lamports: u64,
    QuoteClaimHoldingArgs {
        portfolio,
        holding,
        holding_balance,
        protocol_fees: pf_ratios,
        pool,
    }: &QuoteClaimHoldingArgs,
) -> Result<ClaimHoldingQuote, QuoteRebalErr<Infallible>> {
    // sanity check: single holding should be <= portfolio
    if *holding.sol_value() > *portfolio.sol_value()
        || *holding.outstanding() > *portfolio.outstanding()
    {
        return Err(QuoteRebalErr::Math);
    }

    if claim_lamports > *portfolio.outstanding() {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(claim_lamports, *holding.outstanding()),
        ));
    }

    // check pool empty
    let liq_avail = match pool.liq_avail_checked() {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    if liq_avail > 0 {
        return Err(QuoteRebalErr::RebalExceedLimit(
            OutOfRangeErrU64::actual_exceed_max(claim_lamports, 0),
        ));
    }

    if *holding.outstanding() == 0 || *holding.sol_value() == 0 || *holding_balance == 0 {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(claim_lamports, 0),
        ));
    }

    // since liq_avail is 0, there is no usable sol in the pool account
    // - pool's total sol deposits = portfolio.outstanding
    // - pool's total sol value = portfolio.sol_value
    //
    // user contributed `r = claim_lamports / portfolio.outstanding` to the pool,
    // entitled to `r * portfolio.sol_value` lamports worth of holding tokens
    // before fees

    let out_lamports_no_pf = match Floor(Ratio {
        n: claim_lamports,
        d: *portfolio.outstanding(),
    })
    .apply(*portfolio.sol_value())
    {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let protocol_fees = match calc_protocol_fees(out_lamports_no_pf, portfolio, pf_ratios) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let protocol_fee_lamports = match protocol_fees.total_checked() {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };

    // true in cases of extreme loss of SOL value
    if protocol_fee_lamports > out_lamports_no_pf {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(protocol_fee_lamports, out_lamports_no_pf),
        ));
    }

    let out_lamports = match out_lamports_no_pf.checked_sub(protocol_fee_lamports) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let out = match Floor(Ratio {
        n: out_lamports,
        d: *holding.sol_value(),
    })
    .apply(*holding_balance)
    {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let outstanding_dec = match (claim_lamports as i128).checked_sub(protocol_fee_lamports as i128)
    {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    Ok(ClaimHoldingQuote {
        qtys: RebalInpOut::const_from_destr(RebalQtysDestr {
            amt: InpOut::const_from_destr(InpOutDestr {
                inp: claim_lamports,
                out,
            }),
            sol_value: InpOut::const_from_destr(InpOutDestr {
                inp: claim_lamports,
                out: out_lamports,
            }),
        }),
        outstanding_dec,
        protocol_fees,
    })
}

#[cfg(kani)]
mod ver {
    use super::{quote_claim_holding, ClaimHoldingQuote, QuoteClaimHoldingArgs};
    use crate::{
        accounts::{
            PoolLiqLamportVals, PoolLiqLamportsDestr, PoolV1LamportVals, PoolV1LamportsDestr,
            ProtocolV1FeeRatios, ProtocolV1FeesDestr,
        },
        typedefs::{HoldingV1LamportVals, HoldingV1LamportsDestr, Nanos},
        utils::{protocol_fee_ratio, spec::is_pool_dep_solvent},
    };

    fn valid_claim_holding_args() -> (u64, QuoteClaimHoldingArgs, PoolV1LamportVals, u64, u64) {
        #[rustfmt::skip]
        let [
            outstanding,
            sol_value,
            portfolio_outstanding,
            portfolio_sol_value,
            holding_balance,
            claim_lamports,
            rent_exempt,
            pool_outstanding,
            pool_protocol_fee,
            pool_acc_lamports,
            mint_supply
        ] = [kani::any::<u64>(); 11];
        let [fixed_fee_nanos, perf_fee_nanos] = [kani::any::<u32>(); 2];

        // tight bounds needed: Ceil::apply uses u128 math which creates huge SAT formulas.
        // This also allows us to assume that no overflow occurs during quote()
        const TIGHT_LAMPORT_BOUND: u64 = u32::MAX as u64;
        kani::assume(outstanding > 0 && outstanding <= TIGHT_LAMPORT_BOUND);
        kani::assume(sol_value > 0 && sol_value <= TIGHT_LAMPORT_BOUND);

        // this holding is a single entry in the entire portfolio
        kani::assume(
            portfolio_outstanding >= outstanding && portfolio_outstanding <= TIGHT_LAMPORT_BOUND,
        );
        kani::assume(
            portfolio_sol_value >= sol_value && portfolio_sol_value <= TIGHT_LAMPORT_BOUND,
        );

        kani::assume(holding_balance > 0 && holding_balance <= TIGHT_LAMPORT_BOUND);

        kani::assume(claim_lamports > 0 && claim_lamports <= outstanding);
        kani::assume(fixed_fee_nanos < 1_000_000_000);
        kani::assume(perf_fee_nanos < 1_000_000_000);

        // pool-level params for solvency checks
        kani::assume(pool_outstanding <= TIGHT_LAMPORT_BOUND);
        kani::assume(pool_protocol_fee <= TIGHT_LAMPORT_BOUND);
        kani::assume(pool_acc_lamports >= rent_exempt && pool_acc_lamports <= TIGHT_LAMPORT_BOUND);

        let pool_lamports = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt,
            outstanding: pool_outstanding,
            protocol_fee: pool_protocol_fee,
        });
        kani::assume(is_pool_dep_solvent(
            (pool_lamports, pool_acc_lamports),
            mint_supply,
        ));

        let holding = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        });
        let portfolio = HoldingV1LamportVals::const_from_destr(HoldingV1LamportsDestr {
            outstanding: portfolio_outstanding,
            sol_value: portfolio_sol_value,
        });
        let protocol_fees = ProtocolV1FeeRatios::const_from_destr(ProtocolV1FeesDestr {
            fixed: protocol_fee_ratio(Nanos::new(fixed_fee_nanos).unwrap()),
            perf: protocol_fee_ratio(Nanos::new(perf_fee_nanos).unwrap()),
        });
        let pool = PoolLiqLamportVals::const_from_destr(PoolLiqLamportsDestr {
            total: rent_exempt,
            rent_exempt,
        });

        (
            claim_lamports,
            QuoteClaimHoldingArgs {
                portfolio,
                holding,
                holding_balance,
                protocol_fees,
                pool,
            },
            pool_lamports,
            pool_acc_lamports,
            mint_supply,
        )
    }

    /// surplus preserved: outstanding_dec = claim_lamports - protocol_fee_lamports
    #[kani::proof]
    fn claim_holding_preserves_surplus() {
        let (claim_lamports, args, _pool, _acc, _mint) = valid_claim_holding_args();

        let quote = quote_claim_holding(claim_lamports, &args).unwrap();

        let protocol_fee_lamports = quote.protocol_fees.total();
        let expected_outstanding_dec = (claim_lamports as i128) - (protocol_fee_lamports as i128);
        assert_eq!(quote.outstanding_dec, expected_outstanding_dec);
    }

    /// output tokens bounded by holding balance
    #[kani::proof]
    #[kani::unwind(2)]
    fn claim_holding_output_bounded() {
        let (claim_lamports, args, _pool, _acc, _mint) = valid_claim_holding_args();

        let quote = quote_claim_holding(claim_lamports, &args).unwrap();

        assert!(*quote.qtys.amt().out() <= args.holding_balance);
    }

    /// input amounts correctly preserved
    #[kani::proof]
    #[kani::unwind(2)]
    fn claim_holding_input_preserved() {
        let (claim_lamports, args, _pool, _acc, _mint) = valid_claim_holding_args();

        let quote = quote_claim_holding(claim_lamports, &args).unwrap();

        assert_eq!(*quote.qtys.amt().inp(), claim_lamports);
        assert_eq!(*quote.qtys.sol_value().inp(), claim_lamports);
    }

    /// After a claim_holding, the pool remains deposit-solvent:
    /// mint_supply stays ≤ dep_due despite decreasing outstanding.
    #[kani::proof]
    fn pool_remains_dep_solvent_after_claim_holding() {
        let (claim_lamports, args, pool, pool_acc, mint_supply) = valid_claim_holding_args();

        let ClaimHoldingQuote {
            outstanding_dec,
            protocol_fees,
            ..
        } = quote_claim_holding(claim_lamports, &args).unwrap();

        // outstanding_dec is i128; negative means outstanding increases (edge case:
        // protocol_fees > claim_lamports), positive means outstanding decreases.
        let new_outstanding = if outstanding_dec < 0 {
            pool.outstanding() + u64::try_from(-outstanding_dec).unwrap()
        } else {
            pool.outstanding() - u64::try_from(outstanding_dec).unwrap()
        };
        let pf_inc = protocol_fees.total();
        let new_protocol_fee = pool.protocol_fee() + pf_inc;

        let new_pool = PoolV1LamportVals::const_from_destr(PoolV1LamportsDestr {
            rent_exempt: *pool.rent_exempt(),
            outstanding: new_outstanding,
            protocol_fee: new_protocol_fee,
        });

        assert!(
            is_pool_dep_solvent((new_pool, pool_acc), mint_supply),
            "pool must remain dep-solvent after claim_holding"
        );
    }
}

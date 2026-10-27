use core::convert::Infallible;

use sanctum_u64_ratio::{Ceil, Floor, Ratio};

use crate::{
    accounts::{PoolLiqLamportVals, ProtocolV1FeeLamports, ProtocolV1FeeRatios},
    err::OutOfRangeErrU64,
    rebal::{QuoteRebalErr, RebalInpOut, RebalQtysDestr},
    typedefs::HoldingV1LamportVals,
    utils::{calc_protocol_fees, InpOut, InpOutDestr},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuoteClaimHoldingArgs {
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
/// - Returns [`QuoteRebalErr::NotEnoughLiquidity`] if holding does not have
///   enough SOL value to cover protocol fees. This means pools can be bricked by extreme
///   loss of SOL value events. Require external donors to make pool solvent again in those cases.
#[inline]
pub const fn quote_claim_holding(
    claim_lamports: u64,
    QuoteClaimHoldingArgs {
        holding,
        holding_balance,
        protocol_fees: pf_ratios,
        pool,
    }: &QuoteClaimHoldingArgs,
) -> Result<ClaimHoldingQuote, QuoteRebalErr<Infallible>> {
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

    if claim_lamports > *holding.outstanding() {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(claim_lamports, *holding.outstanding()),
        ));
    }
    let usr_share = Ratio {
        n: claim_lamports,
        d: *holding.outstanding(),
    };
    let protocol_fees = match calc_protocol_fees(claim_lamports, holding, pf_ratios) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let protocol_fee_lamports = match protocol_fees.total_checked() {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };

    // true in cases of extreme loss of SOL value
    if protocol_fee_lamports > *holding.sol_value() {
        return Err(QuoteRebalErr::NotEnoughLiquidity(
            OutOfRangeErrU64::actual_exceed_max(protocol_fee_lamports, *holding.sol_value()),
        ));
    }

    // Have to convert protocol fees to in terms of holding,
    // instead of subtracting lamport value straight from claim_lamports
    // because in the case of extreme SOL gains,
    // protocol_fee_lamports > claim_lamports
    let protocol_fee_holding = match Ceil(Ratio {
        n: protocol_fee_lamports,
        d: *holding.sol_value(),
    })
    .apply(*holding_balance)
    {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let out_no_pf = match Floor(usr_share).apply(*holding_balance) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let out = match out_no_pf.checked_sub(protocol_fee_holding) {
        // in cases of extreme loss of SOL value,
        // out_no_pf < protocol_fee_holding
        None => {
            return Err(QuoteRebalErr::NotEnoughLiquidity(
                OutOfRangeErrU64::actual_exceed_max(protocol_fee_holding, out_no_pf),
            ));
        }
        Some(x) => x,
    };
    // Floor the SOL value of the output tokens in favour of the pool
    let out_sol_value = match Floor(Ratio {
        n: out,
        d: *holding_balance,
    })
    .apply(*holding.sol_value())
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
                out: out_sol_value,
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
        rebal::QuoteRebalErr,
        typedefs::{HoldingV1LamportVals, HoldingV1LamportsDestr, Nanos},
        utils::{protocol_fee_ratio, spec::is_pool_dep_solvent},
    };

    fn valid_claim_holding_args() -> (u64, QuoteClaimHoldingArgs, PoolV1LamportVals, u64, u64) {
        let outstanding: u64 = kani::any();
        let sol_value: u64 = kani::any();
        let holding_balance: u64 = kani::any();
        let claim_lamports: u64 = kani::any();
        let rent_exempt: u64 = kani::any();
        let pool_outstanding: u64 = kani::any();
        let pool_protocol_fee: u64 = kani::any();
        let pool_acc_lamports: u64 = kani::any();
        let mint_supply: u64 = kani::any();
        let fixed_fee_nanos: u32 = kani::any();
        let perf_fee_nanos: u32 = kani::any();

        // tight bounds needed: Ceil::apply uses u128 math which creates huge SAT formulas.
        // This also allows us to assume that no overflow occurs during quote()
        const TIGHT_LAMPORT_BOUND: u64 = u32::MAX as u64;
        kani::assume(outstanding > 0 && outstanding <= TIGHT_LAMPORT_BOUND);
        kani::assume(sol_value > 0 && sol_value <= TIGHT_LAMPORT_BOUND);

        kani::assume(holding_balance > 0 && holding_balance <= TIGHT_LAMPORT_BOUND);

        kani::assume(claim_lamports > 0 && claim_lamports <= outstanding);
        kani::assume(fixed_fee_nanos < 1_000_000_000);
        kani::assume(perf_fee_nanos < 1_000_000_000);

        // pool-level params for solvency checks
        kani::assume(outstanding <= pool_outstanding);
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

        let quote = match quote_claim_holding(claim_lamports, &args) {
            Ok(quote) => quote,
            // Fees can exceed the available holding tokens after losses or rounding.
            Err(QuoteRebalErr::NotEnoughLiquidity(_)) => return,
            Err(_) => panic!("unexpected quote error"),
        };

        let protocol_fee_lamports = quote.protocol_fees.total();
        let expected_outstanding_dec = (claim_lamports as i128) - (protocol_fee_lamports as i128);
        assert_eq!(quote.outstanding_dec, expected_outstanding_dec);
    }

    /// output tokens bounded by holding balance
    #[kani::proof]
    #[kani::unwind(2)]
    fn claim_holding_output_bounded() {
        let (claim_lamports, args, _pool, _acc, _mint) = valid_claim_holding_args();

        let quote = match quote_claim_holding(claim_lamports, &args) {
            Ok(quote) => quote,
            Err(QuoteRebalErr::NotEnoughLiquidity(_)) => return,
            Err(_) => panic!("unexpected quote error"),
        };

        assert!(*quote.qtys.amt().out() <= args.holding_balance);
    }

    /// input amounts correctly preserved
    #[kani::proof]
    #[kani::unwind(2)]
    fn claim_holding_input_preserved() {
        let (claim_lamports, args, _pool, _acc, _mint) = valid_claim_holding_args();

        let quote = match quote_claim_holding(claim_lamports, &args) {
            Ok(quote) => quote,
            Err(QuoteRebalErr::NotEnoughLiquidity(_)) => return,
            Err(_) => panic!("unexpected quote error"),
        };

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
        } = match quote_claim_holding(claim_lamports, &args) {
            Ok(quote) => quote,
            Err(QuoteRebalErr::NotEnoughLiquidity(_)) => return,
            Err(_) => panic!("unexpected quote error"),
        };

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

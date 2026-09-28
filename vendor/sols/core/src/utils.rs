use generic_array_struct::generic_array_struct;
use sanctum_fee_ratio::Fee;
use sanctum_u64_ratio::{Ceil, Ratio};

use crate::{
    accounts::{ProtocolV1FeeLamports, ProtocolV1FeeRatios, ProtocolV1Fees, ProtocolV1FeesDestr},
    err::OutOfRangeErrU64,
    internal_utils::{impl_asref, impl_memset},
    typedefs::{HoldingV1LamportVals, Nanos},
};

/// Params to [`lamport_surplus`]
#[generic_array_struct(all pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LamportSurplusParams<T> {
    /// pool's SOLS mint supply
    pub mint_supply: T,

    /// pool's total lamports due to depositors.
    /// See [`crate::accounts::PoolV1Acc::dep_due`]
    pub dep_due: T,
}

pub type LamportSurplusArgs = LamportSurplusParams<u64>;

/// # Returns
/// - Err: [`OutOfRangeErrU64`] (not enough liquidity)
#[inline]
pub const fn lamport_surplus(args: &LamportSurplusArgs) -> Result<u64, OutOfRangeErrU64> {
    match args.dep_due().checked_sub(*args.mint_supply()) {
        None => Err(OutOfRangeErrU64::actual_exceed_max(
            *args.mint_supply(),
            *args.dep_due(),
        )),
        Some(x) => Ok(x),
    }
}

/// Verify and compute amount of SOLS tokens to mint or amount of lamports to claim.
/// Both qtys are equivalent
///
/// # Returns
/// - Ok: Amount. This will be the maximum amount mintable/claimable if `amt == None`.
/// - Err: [`OutOfRangeErrU64`] (not enough liquidity)
#[inline]
pub const fn vercomp_mint_claim_amt(
    amt: Option<u64>,
    args: &LamportSurplusArgs,
) -> Result<u64, OutOfRangeErrU64> {
    let surplus = match lamport_surplus(args) {
        Err(e) => return Err(e),
        Ok(x) => x,
    };
    match amt {
        None => Ok(surplus),
        Some(amt) => {
            if amt > surplus {
                Err(OutOfRangeErrU64::actual_exceed_max(amt, surplus))
            } else {
                Ok(amt)
            }
        }
    }
}

pub type ProtocolFeeRatio = Fee<Ceil<Ratio<u32, u32>>>;

pub const fn protocol_fee_ratio(fee: Nanos) -> ProtocolFeeRatio {
    // safety: Nanos <= 1.0 by construction
    unsafe { ProtocolFeeRatio::new_unchecked(fee.into_ratio()) }
}

/// Returns `None` on overflow
pub const fn calc_protocol_fees(
    lamports_returning: u64,
    holding: &HoldingV1LamportVals,
    fee: &ProtocolV1FeeRatios,
) -> Option<ProtocolV1FeeLamports> {
    let fixed_fee = match fee.fixed().apply(lamports_returning) {
        None => return None,
        Some(x) => x.fee(),
    };
    let ratio = Ratio {
        n: lamports_returning,
        d: *holding.outstanding(),
    };
    let gains = match Ceil(ratio).apply(holding.sol_value().saturating_sub(*holding.outstanding()))
    {
        None => return None,
        Some(x) => x,
    };
    let perf_fee = match fee.perf().apply(gains) {
        None => return None,
        Some(x) => x.fee(),
    };
    Some(ProtocolV1Fees::const_from_destr(ProtocolV1FeesDestr {
        fixed: fixed_fee,
        perf: perf_fee,
    }))
}

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InpOut<T> {
    pub inp: T,
    pub out: T,
}

impl_memset!(InpOut);
impl_asref!(InpOut);

#[cfg(kani)]
pub mod spec {
    use crate::{
        accounts::{
            spec::{is_pool_solvent, solvent_pool},
            PoolV1LamportVals, ProtocolV1FeeRatios, PROTOCOL_V1_MAX_FIXED_FEE_NANOS,
        },
        typedefs::spec::any_nanos,
        utils::{vercomp_mint_claim_amt, LamportSurplusArgs},
    };

    use super::*;

    /// A pool that is solvent for depositors
    /// - satisfies [`is_pool_solvent`]
    /// - has mint_supply <= dep_due_lamports
    ///
    /// Returns `(rent_exempt_pool_lamports(), mint_supply)`
    pub fn is_pool_dep_solvent(
        (pool, pool_lamports): (PoolV1LamportVals, u64),
        mint_supply: u64,
    ) -> bool {
        if !is_pool_solvent((pool, pool_lamports)) {
            return false;
        }
        let dd = pool.dep_due_checked(pool_lamports).unwrap();
        mint_supply <= dd
    }

    /// A `(pool, mint_supply)` that satisfies [`is_pool_dep_solvent`]
    pub fn dep_solvent_pool() -> ((PoolV1LamportVals, u64), u64) {
        let p = solvent_pool();
        let sup = kani::any();
        kani::assume(is_pool_dep_solvent(p, sup));
        (p, sup)
    }

    /// Assumes:
    /// - `pool` is dep solvent
    pub fn is_mint_claim_amt_within_surplus(amt: Option<u64>, pool: &LamportSurplusArgs) -> bool {
        vercomp_mint_claim_amt(amt, pool).is_ok()
    }

    pub fn any_protocol_v1_fee_ratios() -> ProtocolV1FeeRatios {
        let res = ProtocolV1Fees(core::array::from_fn(|_| any_nanos()));
        kani::assume(*res.fixed() <= PROTOCOL_V1_MAX_FIXED_FEE_NANOS);
        ProtocolV1Fees(res.0.map(protocol_fee_ratio))
    }
}

#[cfg(kani)]
mod ver {
    use super::spec::*;
    use super::*;

    /// The onchain prog uses `vercomp_mint_claim_amt` to calculate how much
    /// amt to mint/claim.
    ///
    /// Prove:
    /// (solvent pool & vercomp_mint_claim_amt success) ->
    /// pool remains solvent for depositors after mint/claim of returned amt
    /// &
    /// (solvent pool & vercomp_mint_claim_amt fail) ->
    /// requested_amt > pool_surplus
    #[kani::proof]
    fn pool_remains_dep_solvent_after_mint_claim() {
        let ((p, l), mint_supply) = dep_solvent_pool();
        let dep_due = p.dep_due_checked(l).unwrap();

        let amt = kani::any();
        let args = LamportSurplusParams::from_destr(LamportSurplusParamsDestr {
            mint_supply,
            dep_due,
        });
        let res = vercomp_mint_claim_amt(amt, &args);

        let mc_amt = match (amt, res) {
            (None, Ok(x)) => x,
            (None, Err(_)) => panic!(),
            (Some(x), Ok(y)) => {
                assert_eq!(x, y);
                x
            }
            (Some(x), Err(_)) => {
                let surplus = dep_due.saturating_sub(mint_supply);
                assert!(x > surplus);
                return;
            }
        };

        // mint case
        let new_sup = mint_supply + mc_amt;
        assert!(is_pool_dep_solvent((p, l), new_sup));

        // claim case
        let new_dd = dep_due - mc_amt;
        // `l - mc_amt` may overflow.
        // Program has further assertions on l >= mc_amt
        assert!(new_dd >= mint_supply);
    }
}

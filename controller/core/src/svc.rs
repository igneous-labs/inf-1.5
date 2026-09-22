//! Calculation of SOL value / redemption rate of the LP token (INF)

use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::{SolValCalc, SolValCalcAccs};
use sanctum_u64_ratio::{Floor, Ratio};

use crate::{
    accounts::pool_state::PoolStateV2,
    err::Inf1CtlErr,
    typedefs::pool_sv::{PoolSv, PoolSvLamports},
    yields::release::{ReleaseYield, ReleaseYieldParams},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfCalc {
    pub pool_lamports: PoolSvLamports,
    pub mint_supply: u64,
}

impl InfCalc {
    pub const DEFAULT: Self = Self {
        pool_lamports: PoolSvLamports::memset(0),
        mint_supply: 0,
    };
}

impl Default for InfCalc {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl InfCalc {
    #[inline]
    pub const fn new(pool_state_v2: &PoolStateV2, inf_mint_supply: u64) -> Self {
        Self {
            pool_lamports: PoolSvLamports::from_pool_state_v2(pool_state_v2),
            mint_supply: inf_mint_supply,
        }
    }

    /// Returns what `self` would be in the future given the yield release event
    ///
    /// # Returns
    /// `None` if `apply_yrel` returns None
    #[inline]
    pub fn lookahead(mut self, params: ReleaseYieldParams) -> Option<Self> {
        let yrel = ReleaseYield {
            params,
            withheld_lamports: *self.pool_lamports.withheld(),
        }
        .calc();
        PoolSv(self.pool_lamports.0.each_mut()).apply_yrel(yrel)?;
        Some(self)
    }

    /// Preserve same behaviour of edge-cases as v1:
    /// Calculate lp_token:sol_value at 1:1 exchange rate if
    ///
    /// - LP supply = 0.
    ///   If LP due sol value is not zero then this results in the first LP
    ///   geting all the orphaned sol value in the pool.  
    ///
    /// - LP due sol value = 0 & withheld = 0.
    ///   This results in the entering LP being immediately diluted by
    ///   existing INF token holders if the supply is nonzero.
    ///   Should never happen unless on catastrophic losses.
    ///   If withheld is nonzero, then zero will be returned;
    ///   value should be nonzero again after waiting some time.
    #[inline]
    pub const fn inf_to_sol(&self, inf: u64) -> Option<u64> {
        let r = match self.lp_due_over_supply() {
            None => return None,
            Some(r) => r,
        };
        r.apply(inf)
    }

    /// Edge cases: same as [`Self::inf_to_sol`]
    #[inline]
    pub const fn sol_to_inf(&self, sol: u64) -> Option<RangeInclusive<u64>> {
        let r = match self.lp_due_over_supply() {
            None => return None,
            Some(r) => r,
        };
        let range = match r.reverse_est(sol) {
            None => return None,
            Some(x) => x,
        };
        Some(if *range.start() > *range.end() {
            *range.end()..=*range.start()
        } else {
            range
        })
    }

    /// # Returns
    /// (lamports_due_to_lp / inf_supply), with the following special-cases
    /// that returns 1.0:
    /// - LP supply = 0
    /// - LP due sol value = 0 & withheld = 0
    ///
    /// See doc on [`Self::inf_to_sol`] on why.
    ///
    /// `None` if pool is insolvent for LPers (lp_due is negative)
    #[inline]
    pub const fn lp_due_over_supply(&self) -> Option<Floor<Ratio<u64, u64>>> {
        let lp_due = match self.pool_lamports.lp_due_checked() {
            None => return None,
            Some(x) => x,
        };
        let r = if self.mint_supply == 0 || (lp_due == 0 && *self.pool_lamports.withheld() == 0) {
            Ratio::<u64, u64>::ONE
        } else {
            Ratio {
                n: lp_due,
                d: self.mint_supply,
            }
        };
        Some(Floor(r))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub enum InfCalcErr {
    #[default]
    Math,
}

impl Display for InfCalcErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Math => f.write_str("MathError"),
        }
    }
}

impl Error for InfCalcErr {}

impl From<InfCalcErr> for Inf1CtlErr {
    fn from(e: InfCalcErr) -> Self {
        match e {
            InfCalcErr::Math => Self::MathError,
        }
    }
}

/// SolValCalc traits const adapters
impl InfCalc {
    #[inline]
    pub const fn svc_lst_to_sol(&self, inf: u64) -> Result<RangeInclusive<u64>, InfCalcErr> {
        match self.inf_to_sol(inf) {
            None => Err(InfCalcErr::Math),
            Some(x) => Ok(x..=x),
        }
    }

    #[inline]
    pub const fn svc_sol_to_lst(&self, sol: u64) -> Result<RangeInclusive<u64>, InfCalcErr> {
        match self.sol_to_inf(sol) {
            None => Err(InfCalcErr::Math),
            Some(x) => Ok(x),
        }
    }
}

/// # Notes
/// - the values returned by this SolValCalc do not take into account withdrawal (removeLiquidity) fees,
///   unlike the other stake pools' ones
impl SolValCalc for InfCalc {
    type Error = InfCalcErr;

    #[inline]
    fn lst_to_sol(&self, inf: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(inf)
    }

    #[inline]
    fn sol_to_lst(&self, lamports: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports)
    }
}

/// The INF program does NOT implement the SOL value calculator program interface.
///
/// We also do not need to pass in any additional calc accounts for getting its sol value
/// since all swap instructions that involve minting/burning INF would already include
/// the required 2 accounts, PoolState and INF mint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InfDummyCalcAccs;

impl InfDummyCalcAccs {
    #[inline]
    pub const fn svc_suf_keys_owned(&self) -> <Self as SolValCalcAccs>::KeysOwned {
        []
    }

    #[inline]
    pub const fn svc_suf_is_writer(&self) -> <Self as SolValCalcAccs>::AccFlags {
        []
    }

    #[inline]
    pub const fn svc_suf_is_signer(&self) -> <Self as SolValCalcAccs>::AccFlags {
        []
    }
}

impl SolValCalcAccs for InfDummyCalcAccs {
    type KeysOwned = [[u8; 32]; 0];

    type AccFlags = [bool; 0];

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.svc_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.svc_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.svc_suf_is_signer()
    }
}

/// The standalone `inf-svc` SOL value calculator program.
///
/// [`InfDummyCalcAccs`] is the controller pricing its *own* LP token natively
/// (zero suffix). This program is the same [`InfCalc`] math packaged as a
/// generic SVC program, so a *different* controller -- e.g. reserve-v2 -- can
/// price the INF token held as one of its LSTs. It exposes the generic
/// interface's four-account suffix.
pub const INF_SVC_PROGRAM_ID_STR: &str = "1nf7dspGYz1CTALJbtNgjvcSYWiFz5N3c2EuUZLSWCL";
pub const INF_SVC_PROGRAM_ID: [u8; 32] = const_crypto::bs58::decode_pubkey(INF_SVC_PROGRAM_ID_STR);

/// The controller whose pool this program prices: always INF, never the
/// feature-gated `CONST_KEYS_OWNED.program()`, since this program exists to
/// price INF for a *different* controller.
pub const INF_SVC_POOL_PROG_ID: [u8; 32] = crate::keys::INF_PROGRAM_ID;

const INF_SVC_STATE_SEED: [u8; 5] = *b"state";
const INF_SVC_STATE: ([u8; 32], u8) =
    const_crypto::ed25519::derive_program_address(&[&INF_SVC_STATE_SEED], &INF_SVC_PROGRAM_ID);
pub const INF_SVC_STATE_ID: [u8; 32] = INF_SVC_STATE.0;
pub const INF_SVC_STATE_BUMP: u8 = INF_SVC_STATE.1;

const BPF_LOADER_V3: [u8; 32] =
    const_crypto::bs58::decode_pubkey("BPFLoaderUpgradeab1e11111111111111111111111");
const INF_SVC_POOL_PROGDATA: ([u8; 32], u8) =
    const_crypto::ed25519::derive_program_address(&[&INF_SVC_POOL_PROG_ID], &BPF_LOADER_V3);
pub const INF_SVC_POOL_PROGDATA_ID: [u8; 32] = INF_SVC_POOL_PROGDATA.0;
pub const INF_SVC_POOL_PROGDATA_BUMP: u8 = INF_SVC_POOL_PROGDATA.1;

const INF_SVC_POOL_STATE: ([u8; 32], u8) = crate::pda::const_find_pool_state(&INF_SVC_POOL_PROG_ID);
pub const INF_SVC_POOL_STATE_ID: [u8; 32] = INF_SVC_POOL_STATE.0;
pub const INF_SVC_POOL_STATE_BUMP: u8 = INF_SVC_POOL_STATE.1;

/// [`InfExtCalcAccs`]'s account suffix, in the generic SVC interface's order:
/// `[state, pool_state, pool_prog, pool_progdata]`.
///
/// All four are const: the `inf-svc` program verifies `pool_state` against the
/// `pool_prog` baked into its deployment, so unlike the SPL calculators the pool
/// state is not a runtime choice -- a different pool state would be a different
/// program, with all four differing together.
mod inf_ext {
    use inf1_svc_generic::instructions::interface::{
        IxSufAccFlags, IxSufAccsDestr, IxSufKeysOwned, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER,
    };

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InfExtCalcAccs;

    impl InfExtCalcAccs {
        pub const KEYS_OWNED: IxSufKeysOwned = IxSufKeysOwned::const_from_destr(IxSufAccsDestr {
            state: INF_SVC_STATE_ID,
            pool_state: INF_SVC_POOL_STATE_ID,
            pool_prog: INF_SVC_POOL_PROG_ID,
            pool_progdata: INF_SVC_POOL_PROGDATA_ID,
        });

        #[inline]
        pub const fn svc_suf_keys_owned(&self) -> IxSufKeysOwned {
            Self::KEYS_OWNED
        }

        #[inline]
        pub const fn svc_suf_is_writer(&self) -> IxSufAccFlags {
            IX_SUF_IS_WRITER
        }

        #[inline]
        pub const fn svc_suf_is_signer(&self) -> IxSufAccFlags {
            IX_SUF_IS_SIGNER
        }
    }

    impl SolValCalcAccs for InfExtCalcAccs {
        type KeysOwned = IxSufKeysOwned;

        type AccFlags = IxSufAccFlags;

        #[inline]
        fn suf_keys_owned(&self) -> Self::KeysOwned {
            self.svc_suf_keys_owned()
        }

        #[inline]
        fn suf_is_writer(&self) -> Self::AccFlags {
            self.svc_suf_is_writer()
        }

        #[inline]
        fn suf_is_signer(&self) -> Self::AccFlags {
            self.svc_suf_is_signer()
        }
    }
}
pub use inf_ext::*;

#[cfg(test)]
mod tests {
    use std::convert::identity;

    use proptest::prelude::*;

    use crate::typedefs::pool_sv::test_utils::pool_sv_lamports_invar_strat;

    use super::*;

    /// - PoolSvLamports solvency invariant respected
    fn any_calc() -> impl Strategy<Value = InfCalc> {
        (
            any::<u64>(),
            any::<u64>()
                .prop_map(pool_sv_lamports_invar_strat)
                .prop_flat_map(identity),
        )
            .prop_map(|(mint_supply, pool_lamports)| InfCalc {
                pool_lamports,
                mint_supply,
            })
    }

    proptest! {
        #[test]
        fn lp_due_over_supply_zero_cases(calc in any_calc()) {
            let r = calc.lp_due_over_supply().unwrap();

            // this is the only case where r should be 0
            if *calc.pool_lamports.withheld() != 0
                && calc.pool_lamports.lp_due_checked().unwrap() == 0
            {
                assert!(r.0.is_zero());
            } else {
                assert!(!r.0.is_zero());
            }
        }
    }

    // make sure we've handled the `Ratio::reverse_est`
    // case correctly
    fn assert_valid_range(r: &RangeInclusive<u64>) {
        assert!(r.start() <= r.end(), "{r:?}");
    }

    fn assert_err_bound(val: u64, r: &RangeInclusive<u64>) {
        assert!(r.contains(&val), "{val} {r:?}");
    }

    proptest! {
        #[test]
        fn sol_to_inf_rt_errbound(
            val: u64,
            calc in any_calc(),
        ) {
            if let Some(inf) = calc.sol_to_inf(val) {
                let [min_rt, max_rt] =
                    [inf.start(), inf.end()].map(|x| calc.inf_to_sol(*x).unwrap());
                assert_err_bound(val, &(min_rt..=max_rt));
                assert_valid_range(&inf);
            }
            if let Some(sol) = calc.inf_to_sol(val) {
                let rt = calc.sol_to_inf(sol).unwrap();
                assert_err_bound(val, &rt);
                assert_valid_range(&rt);
            }
        }
    }
}

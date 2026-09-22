use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use inf1_ctl_core::{
    accounts::pool_state::{PoolStateV2, VerPoolState},
    err::Inf1CtlErr,
    pda::CONST_PDA_KEYS_OWNED,
    svc::{InfCalc, InfDummyCalcAccs, InfExtCalcAccs},
    yields::release::ReleaseYieldParams,
};
use inf1_svc_std::update::{Account, AccountsToUpdateSvc, UpdateErr, UpdateMap, UpdateSvc};

// Re-exports
pub use inf1_ctl_core::*;

pub const INF_MINT_ID_STR: &str = "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm";
pub const INF_MINT_ID: [u8; 32] = const_crypto::bs58::decode_pubkey(INF_MINT_ID_STR);

/// The clock sysvar, read for the release-yield lookahead the `inf-svc` program
/// applies on chain.
pub const SYSVAR_CLOCK: [u8; 32] =
    const_crypto::bs58::decode_pubkey("SysvarC1ock11111111111111111111111111111111");

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfSvcStd {
    pub calc: InfCalc,

    // FIXME? this mint addr will probably be duplicated in
    // most contexts with the one stored in an accompanying PoolState
    pub mint_addr: [u8; 32],

    pub pool_state_addr: [u8; 32],
}

pub type PkIter = core::array::IntoIter<[u8; 32], 2>;

/// [`InfExtSvcStd`] reads one more account than [`InfSvcStd`]: the clock, for
/// the release-yield lookahead.
pub type InfExtPkIter = core::array::IntoIter<[u8; 32], 3>;

impl AccountsToUpdateSvc for InfSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        self.atus_accs_to_update_svc().into_iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for InfUpdateErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for InfUpdateErr {}

/// [`InfExtSvcStd`]'s update error.
///
/// Wider than [`InfUpdateErr`] because, unlike the controller-native calc, it
/// applies the on-chain program's release-yield lookahead, which can fail with
/// an [`Inf1CtlErr`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfExtUpdateErr {
    AccDeser { pk: [u8; 32] },

    Ctl(Inf1CtlErr),
}

impl Display for InfExtUpdateErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
            Self::Ctl(e) => Display::fmt(e, f),
        }
    }
}

impl Error for InfExtUpdateErr {}

impl From<InfUpdateErr> for InfExtUpdateErr {
    #[inline]
    fn from(InfUpdateErr::AccDeser { pk }: InfUpdateErr) -> Self {
        Self::AccDeser { pk }
    }
}

impl UpdateSvc for InfSvcStd {
    type InnerErr = InfUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.us_update_svc(update_map)
    }
}

impl InfSvcStd {
    pub const DEFAULT: Self = Self {
        calc: InfCalc::DEFAULT,
        mint_addr: [0u8; 32],
        pool_state_addr: *CONST_PDA_KEYS_OWNED.pool_state(),
    };

    #[inline]
    pub const fn atus_accs_to_update_svc(&self) -> [[u8; 32]; 2] {
        [self.pool_state_addr, self.mint_addr]
    }

    #[inline]
    pub fn us_update_svc(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfUpdateErr>> {
        let (pool_state_v2, inf_mint_supply) =
            fetched_pool_state_and_supply(&self.pool_state_addr, &self.mint_addr, update_map)?;
        self.calc = InfCalc::new(&pool_state_v2, inf_mint_supply);
        Ok(())
    }
}

/// Accessors
impl InfSvcStd {
    #[inline]
    pub const fn as_calc(&self) -> &InfCalc {
        &self.calc
    }

    #[inline]
    pub const fn as_accs(&self) -> &InfDummyCalcAccs {
        &InfDummyCalcAccs
    }
}

/// The standalone `inf-svc` program's calculator, as opposed to
/// [`InfSvcStd`]'s controller-native one.
///
/// Same `InfCalc` math, but it carries the generic interface's account suffix
/// ([`InfExtCalcAccs`]) and is identified by the standalone program's ID. Used
/// to price the INF token when it is an LST of a *different* controller, e.g.
/// reserve-v2.
///
/// It wraps [`InfSvcStd`] with the pool state pinned to the INF controller's:
/// `InfSvcStd::DEFAULT` uses the feature-gated controller, which in a
/// reserve-v2 build would point at the reserve-v2 pool.
///
/// Unlike the controller-native calc, its update also reads the clock and
/// applies the program's release-yield lookahead, so an off-chain quote matches
/// what the on-chain program returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfExtSvcStd {
    pub inner: InfSvcStd,

    /// Extra slots to look the yield release forward, *on top of* the clock slot
    /// the on-chain program would use. `0` mirrors the program exactly; a
    /// positive value projects that many slots into the future.
    ///
    /// Deliberately not the same as the std quote API's `slot_lookahead`, which
    /// is relative to `PoolStateV2::last_release_slot` and stands in for the
    /// elapsed count rather than adding to the clock.
    pub slot_lookahead: u64,
}

impl AccountsToUpdateSvc for InfExtSvcStd {
    type PkIter = InfExtPkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        [
            self.inner.pool_state_addr,
            self.inner.mint_addr,
            SYSVAR_CLOCK,
        ]
        .into_iter()
    }
}

impl UpdateSvc for InfExtSvcStd {
    type InnerErr = InfExtUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        let (pool_state_v2, inf_mint_supply) = fetched_pool_state_and_supply(
            &self.inner.pool_state_addr,
            &self.inner.mint_addr,
            &update_map,
        )
        .map_err(|e| e.map_inner(InfExtUpdateErr::from))?;

        // Mirror the program's `try_derive_calc`: look the yield release
        // forward to the current slot (plus this instance's lookahead) before
        // pricing.
        let clock_acc = update_map.get_account_checked(&SYSVAR_CLOCK)?;
        let clock_slot =
            clock_slot(clock_acc.data()).ok_or(UpdateErr::Inner(InfExtUpdateErr::AccDeser {
                pk: SYSVAR_CLOCK,
            }))?;
        let curr_slot = clock_slot
            .checked_add(self.slot_lookahead)
            .ok_or(UpdateErr::Inner(InfExtUpdateErr::Ctl(
                Inf1CtlErr::MathError,
            )))?;
        let params = ReleaseYieldParams::new(&pool_state_v2, curr_slot)
            .map_err(|e| UpdateErr::Inner(InfExtUpdateErr::Ctl(e)))?;

        self.inner.calc = InfCalc::new(&pool_state_v2, inf_mint_supply)
            .lookahead(params)
            .ok_or(UpdateErr::Inner(InfExtUpdateErr::Ctl(
                Inf1CtlErr::MathError,
            )))?;

        Ok(())
    }
}

impl InfExtSvcStd {
    pub const DEFAULT: Self = Self {
        inner: InfSvcStd {
            calc: InfCalc::DEFAULT,
            mint_addr: INF_MINT_ID,
            pool_state_addr: inf1_ctl_core::svc::INF_SVC_POOL_STATE_ID,
        },
        slot_lookahead: 0,
    };

    /// Overrides [`Self::slot_lookahead`].
    #[inline]
    pub const fn with_slot_lookahead(mut self, slot_lookahead: u64) -> Self {
        self.slot_lookahead = slot_lookahead;
        self
    }

    #[inline]
    pub const fn as_calc(&self) -> &InfCalc {
        self.inner.as_calc()
    }

    #[inline]
    pub const fn as_accs(&self) -> &InfExtCalcAccs {
        &InfExtCalcAccs
    }
}

// TODO below util fns are duplicated in inf1-std

/// Reads the pool state and the LP/INF mint supply, the two inputs shared by
/// both INF calculators.
fn fetched_pool_state_and_supply(
    pool_state_addr: &[u8; 32],
    mint_addr: &[u8; 32],
    update_map: impl UpdateMap,
) -> Result<(PoolStateV2, u64), UpdateErr<InfUpdateErr>> {
    let [p, m] = [pool_state_addr, mint_addr].map(|a| update_map.get_account_checked(a));
    let pool_state = VerPoolState::try_from_acc_data(p?.data())
        .ok_or(UpdateErr::Inner(InfUpdateErr::AccDeser {
            pk: *pool_state_addr,
        }))?
        .migrated(0 /* migration_slot has no effect here */);
    let supply = token_supply_from_mint_data(m?.data())
        .ok_or(UpdateErr::Inner(InfUpdateErr::AccDeser { pk: *mint_addr }))?;
    Ok((pool_state, supply))
}

fn clock_slot(clock_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(clock_acc_data, 0)
}

fn token_supply_from_mint_data(mint_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(mint_acc_data, 36)
}

fn u64_le_at(data: &[u8], at: usize) -> Option<u64> {
    chunk_at(data, at).map(|c| u64::from_le_bytes(*c))
}

fn chunk_at<const N: usize>(data: &[u8], at: usize) -> Option<&[u8; N]> {
    data.get(at..).and_then(|s| s.first_chunk())
}

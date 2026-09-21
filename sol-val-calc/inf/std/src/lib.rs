use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use inf1_ctl_core::{
    accounts::pool_state::VerPoolState,
    pda::CONST_PDA_KEYS_OWNED,
    svc::{InfCalc, InfDummyCalcAccs, InfExtCalcAccs},
};
use inf1_svc_std::update::{Account, AccountsToUpdateSvc, UpdateErr, UpdateMap, UpdateSvc};

// Re-exports
pub use inf1_ctl_core::*;

pub const INF_MINT_ID_STR: &str = "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm";
pub const INF_MINT_ID: [u8; 32] = const_crypto::bs58::decode_pubkey(INF_MINT_ID_STR);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfSvcStd {
    pub calc: InfCalc,

    // FIXME? this mint addr will probably be duplicated in
    // most contexts with the one stored in an accompanying PoolState
    pub mint_addr: [u8; 32],

    pub pool_state_addr: [u8; 32],
}

pub type PkIter = core::array::IntoIter<[u8; 32], 2>;

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
        let [pool_addr, mint_addr] = self.atus_accs_to_update_svc();
        let [p, m] = [pool_addr, mint_addr].map(|a| update_map.get_account_checked(&a));
        let pool_state_acc = p?;
        let lp_mint_acc = m?;

        let pool_state_v2 = VerPoolState::try_from_acc_data(pool_state_acc.data())
            .ok_or(UpdateErr::Inner(InfUpdateErr::AccDeser { pk: pool_addr }))?
            .migrated(0 /* migration_slot has no effect here */);

        let inf_mint_supply = token_supply_from_mint_data(lp_mint_acc.data())
            .ok_or(UpdateErr::Inner(InfUpdateErr::AccDeser { pk: mint_addr }))?;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfExtSvcStd(pub InfSvcStd);

impl AccountsToUpdateSvc for InfExtSvcStd {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_svc(&self) -> Self::PkIter {
        self.0.accounts_to_update_svc()
    }
}

impl UpdateSvc for InfExtSvcStd {
    type InnerErr = InfUpdateErr;

    #[inline]
    fn update_svc(&mut self, update_map: impl UpdateMap) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.0.update_svc(update_map)
    }
}

impl InfExtSvcStd {
    pub const DEFAULT: Self = Self(InfSvcStd {
        calc: InfCalc::DEFAULT,
        mint_addr: INF_MINT_ID,
        pool_state_addr: inf1_ctl_core::svc::INF_SVC_POOL_STATE_ID,
    });

    #[inline]
    pub const fn as_calc(&self) -> &InfCalc {
        self.0.as_calc()
    }

    #[inline]
    pub const fn as_accs(&self) -> &InfExtCalcAccs {
        &InfExtCalcAccs
    }
}

// TODO below util fns are duplicated in inf1-std

fn token_supply_from_mint_data(mint_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(mint_acc_data, 36)
}

fn u64_le_at(data: &[u8], at: usize) -> Option<u64> {
    chunk_at(data, at).map(|c| u64::from_le_bytes(*c))
}

fn chunk_at<const N: usize>(data: &[u8], at: usize) -> Option<&[u8; N]> {
    data.get(at..).and_then(|s| s.first_chunk())
}

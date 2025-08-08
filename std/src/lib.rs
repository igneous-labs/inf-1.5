use std::{
    borrow::Borrow,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use inf1_core::inf1_ctl_core::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    typedefs::lst_state::{LstState, LstStatePacked},
};
use inf1_pp_ag_std::PricingProgAg;
use inf1_svc_ag_std::{SvcAg, SvcAgStd, SvcAgTy};

use crate::err::InfErr;

// Re-exports
pub use inf1_core::*;
pub mod svc {
    pub use inf1_svc_ag_std::*;
}
pub mod pp {
    pub use inf1_pp_ag_std::*;
}

pub mod err;
pub mod pda;
pub mod trade;
pub mod update;

mod utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inf<F, C> {
    pub(crate) pool: PoolState,
    pub(crate) lst_state_list_data: Box<[u8]>,

    pub(crate) lp_token_supply: Option<u64>,

    pub(crate) pricing: PricingProgAg<F, C>,

    /// key=mint
    pub(crate) lst_reserves: HashMap<[u8; 32], Reserves>,

    /// key=mint
    pub(crate) lst_calcs: HashMap<[u8; 32], SvcAgStd>,

    /// Map of `spl_lst_mint: spl_stake_pool_addr`
    ///
    /// We store this in the struct so that we are able to
    /// initialize any added SPL LSTs newly added to the pool
    pub(crate) spl_lsts: HashMap<[u8; 32], [u8; 32]>,

    pub(crate) find_pda: F,

    pub(crate) create_pda: C,
}

pub type FindPdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>;

pub type CreatePdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>;

pub type InfStd = Inf<FindPdaFnPtr, CreatePdaFnPtr>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reserves {
    pub balance: u64,
    // TODO: add more Reserves related fields as required
}

/// Constructors
impl<F, C> Inf<F, C> {
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn new(
        pool: PoolState,
        lst_state_list_data: Box<[u8]>,
        lp_token_supply: Option<u64>,
        pricing: PricingProgAg<F, C>,
        lst_reserves: HashMap<[u8; 32], Reserves>,
        lst_calcs: HashMap<[u8; 32], SvcAgStd>,
        spl_lsts: HashMap<[u8; 32], [u8; 32]>,
        find_pda: F,
        create_pda: C,
    ) -> Result<Self, InfErr> {
        if LstStatePackedList::of_acc_data(&lst_state_list_data).is_none() {
            return Err(InfErr::AccDeser {
                pk: inf1_core::inf1_ctl_core::keys::POOL_STATE_ID,
            });
        }

        Ok(Self {
            pool,
            lst_state_list_data,
            lp_token_supply,
            pricing,
            lst_reserves,
            lst_calcs,
            spl_lsts,
            find_pda,
            create_pda,
        })
    }
}

/// Accessors
impl<F, C> Inf<F, C> {
    pub fn lst_state_list(&self) -> &[LstStatePacked] {
        // unwrap-safety: valid list checked at construction and update time
        LstStatePackedList::of_acc_data(&self.lst_state_list_data)
            .unwrap()
            .0
    }

    /// Lazily initializes a LST calculator.
    ///
    /// Replaces the old LST calculator data with fresh default if sol val calc program was
    /// determined to have changed
    ///
    /// Errors if:
    /// - LST is a SPL LST and SPL data is not in `self.spl_lsts`
    /// - SOL value calculator is unknown
    pub fn try_get_or_init_lst_svc_mut<'a>(
        &'a mut self,
        LstState {
            mint,
            sol_value_calculator,
            ..
        }: &LstState,
    ) -> Result<&'a mut SvcAgStd, InfErr> {
        let ty =
            SvcAgTy::try_from_svc_program_id(sol_value_calculator).ok_or(InfErr::UnknownSvc {
                svc_prog_id: *sol_value_calculator,
            })?;

        // Make closure to reuse code below.
        // Below structure uses entry api to work around simultaneous mutable borrow issues
        let init_data_fn = || {
            Ok::<_, InfErr>(match ty {
                SvcAgTy::Lido => SvcAg::Lido(()),
                SvcAgTy::Marinade => SvcAg::Marinade(()),
                SvcAgTy::SanctumSpl => {
                    let stake_pool_addr = self
                        .spl_lsts
                        .get(mint)
                        .ok_or(InfErr::MissingSplData { mint: *mint })?;
                    SvcAg::SanctumSpl(*stake_pool_addr)
                }
                SvcAgTy::SanctumSplMulti => {
                    let stake_pool_addr = self
                        .spl_lsts
                        .get(mint)
                        .ok_or(InfErr::MissingSplData { mint: *mint })?;
                    SvcAg::SanctumSplMulti(*stake_pool_addr)
                }
                SvcAgTy::Spl => {
                    let stake_pool_addr = self
                        .spl_lsts
                        .get(mint)
                        .ok_or(InfErr::MissingSplData { mint: *mint })?;
                    SvcAg::Spl(*stake_pool_addr)
                }
                SvcAgTy::Wsol => SvcAg::Wsol(()),
            })
        };

        Ok(match self.lst_calcs.entry(*mint) {
            Entry::Occupied(mut e) => {
                if e.get().0.ty() != ty {
                    let init_data = init_data_fn()?;
                    e.insert(SvcAgStd::new(init_data));
                }
                e.into_mut()
            }
            Entry::Vacant(e) => {
                let init_data = init_data_fn()?;
                e.insert(SvcAgStd::new(init_data))
            }
        })
    }

    #[inline]
    pub fn try_get_lst_reserves<Q>(&self, lst_mint: &Q) -> Option<&Reserves>
    where
        Q: ?Sized + Hash + Eq,
        [u8; 32]: Borrow<Q>,
    {
        self.lst_reserves.get(lst_mint)
    }

    #[inline]
    pub const fn spl_lsts(&self) -> &HashMap<[u8; 32], [u8; 32]> {
        &self.spl_lsts
    }
}

/// Mutators
impl<F, C> Inf<F, C> {
    #[inline]
    pub fn extend_spl_lsts(&mut self, kv: impl IntoIterator<Item = ([u8; 32], [u8; 32])>) {
        self.spl_lsts.extend(kv)
    }
}

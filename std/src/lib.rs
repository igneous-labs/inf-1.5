use std::collections::{hash_map::Entry, HashMap};

use inf1_core::inf1_ctl_core::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    keys::LST_STATE_LIST_ID,
    typedefs::lst_state::{LstState, LstStatePacked},
};
use inf1_pp_ag_std::PricingProgAg;
use inf1_svc_ag_std::{calc::SvcCalcAg, instructions::SvcCalcAccsAg, SvcAg, SvcAgStd, SvcAgTy};

use crate::{
    err::InfErr,
    utils::{try_default_pricing_prog_from_program_id, try_find_lst_state},
};

// Re-exports
pub use inf1_core::*;
pub use inf1_pp_ag_std;
pub use inf1_svc_ag_std;

pub mod err;
pub mod pda;
pub mod rebalance;
pub mod trade;
pub mod update;

mod utils;

// just make all fields pub to enable destructuring for simultaneous mutable borrow of fields
// for downstream crates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inf<F, C> {
    pub pool: PoolState,

    pub lst_state_list_data: Box<[u8]>,

    pub lp_token_supply: Option<u64>,

    pub pricing: PricingProgAg<F, C>,

    /// key=mint
    pub lst_reserves: HashMap<[u8; 32], Reserves>,

    /// key=mint
    pub lst_calcs: HashMap<[u8; 32], SvcAgStd>,

    /// Map of `spl_lst_mint: spl_stake_pool_addr`
    ///
    /// We store this in the struct so that we are able to
    /// initialize any added SPL LSTs newly added to the pool
    pub spl_lsts: HashMap<[u8; 32], [u8; 32]>,

    pub find_pda: F,

    pub create_pda: C,
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
impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn new(
        pool: PoolState,
        lst_state_list_data: Box<[u8]>,
        lp_token_supply: Option<u64>,
        pricing: Option<PricingProgAg<F, C>>,
        lst_reserves: HashMap<[u8; 32], Reserves>,
        lst_calcs: HashMap<[u8; 32], SvcAgStd>,
        spl_lsts: HashMap<[u8; 32], [u8; 32]>,
        find_pda: F,
        create_pda: C,
    ) -> Result<Self, InfErr> {
        if LstStatePackedList::of_acc_data(&lst_state_list_data).is_none() {
            return Err(InfErr::AccDeser {
                pk: inf1_core::inf1_ctl_core::keys::LST_STATE_LIST_ID,
            });
        }

        let pricing = match pricing {
            Some(p) => p,
            None => try_default_pricing_prog_from_program_id(
                &pool.pricing_program,
                find_pda.clone(),
                create_pda.clone(),
            )?,
        };

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
    #[inline]
    pub fn try_lst_state_list(&self) -> Result<&[LstStatePacked], InfErr> {
        Ok(LstStatePackedList::of_acc_data(&self.lst_state_list_data)
            .ok_or(InfErr::AccDeser {
                pk: LST_STATE_LIST_ID,
            })?
            .0)
    }

    #[inline]
    pub fn try_get_lst_svc(&self, mint: &[u8; 32]) -> Result<&SvcAgStd, InfErr> {
        self.lst_calcs
            .get(mint)
            .ok_or(InfErr::MissingSvcData { mint: *mint })
    }

    /// Lazily initializes a LST calculator.
    ///
    /// Replaces the old LST calculator data with fresh default if sol val calc program was
    /// determined to have changed
    ///
    /// Errors if:
    /// - LST is a SPL LST and SPL data is not in `self.spl_lsts`
    /// - SOL value calculator is unknown
    #[inline]
    pub fn try_get_or_init_lst_svc<'a>(
        &'a mut self,
        lst_state: &LstState,
    ) -> Result<&'a mut SvcAgStd, InfErr> {
        let Self {
            spl_lsts,
            lst_calcs,
            ..
        } = self;
        Self::try_get_or_init_lst_svc_static(lst_calcs, spl_lsts, lst_state)
    }

    // Associated fn format like this so that it can be used by external crates
    // (jup-interface)
    #[inline]
    pub fn try_get_or_init_lst_svc_static<'a>(
        lst_calcs: &'a mut HashMap<[u8; 32], SvcAgStd>,
        spl_lsts: &HashMap<[u8; 32], [u8; 32]>,
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
                SvcAg::Inf(_) => SvcAg::Inf(()),
                SvcAgTy::Lido(_) => SvcAg::Lido(()),
                SvcAgTy::Marinade(_) => SvcAg::Marinade(()),
                SvcAgTy::SanctumSpl(_) => {
                    let stake_pool_addr = spl_lsts
                        .get(mint)
                        .ok_or(InfErr::MissingSplData { mint: *mint })?;
                    SvcAg::SanctumSpl(*stake_pool_addr)
                }
                SvcAgTy::SanctumSplMulti(_) => {
                    let stake_pool_addr = spl_lsts
                        .get(mint)
                        .ok_or(InfErr::MissingSplData { mint: *mint })?;
                    SvcAg::SanctumSplMulti(*stake_pool_addr)
                }
                SvcAgTy::Spl(_) => {
                    let stake_pool_addr = spl_lsts
                        .get(mint)
                        .ok_or(InfErr::MissingSplData { mint: *mint })?;
                    SvcAg::Spl(*stake_pool_addr)
                }
                SvcAgTy::Wsol(_) => SvcAg::Wsol(()),
            })
        };

        Ok(match lst_calcs.entry(*mint) {
            Entry::Occupied(mut e) => {
                // sol val calc program was changed
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

    pub(crate) fn lst_state_and_calc(
        &self,
        mint: &[u8; 32],
    ) -> Result<(LstState, SvcCalcAg), InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.try_lst_state_list()?, mint)?;
        let calc = self
            .try_get_lst_svc(mint)?
            .as_sol_val_calc()
            .ok_or(InfErr::MissingSvcData { mint: *mint })?
            .to_owned_copy();
        Ok((lst_state, calc))
    }

    pub(crate) fn lst_state_and_calc_mut(
        &mut self,
        mint: &[u8; 32],
    ) -> Result<(LstState, SvcCalcAg), InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.try_lst_state_list()?, mint)?;
        let calc = self
            .try_get_or_init_lst_svc(&lst_state)?
            .as_sol_val_calc()
            .ok_or(InfErr::MissingSvcData { mint: *mint })?
            .to_owned_copy();
        Ok((lst_state, calc))
    }
}

/// (lst_index, lst_state, lst_calc_accs, lst_reserves_addr)
pub(crate) type LstVarsTup = (u32, LstState, SvcCalcAccsAg, [u8; 32]);

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    pub(crate) fn reserves_balance_checked(
        &self,
        mint: &[u8; 32],
        lst_state: &LstState,
    ) -> Result<u64, InfErr> {
        Ok(self
            .lst_reserves
            .get(mint)
            .ok_or_else(|| {
                self.create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
                    .map_or_else(|| InfErr::NoValidPda, |pk| InfErr::MissingAcc { pk })
            })?
            .balance)
    }

    pub(crate) fn lst_vars(&self, mint: &[u8; 32]) -> Result<LstVarsTup, InfErr> {
        let (i, lst_state) = try_find_lst_state(self.try_lst_state_list()?, mint)?;
        let calc_accs = self
            .try_get_lst_svc(mint)?
            .as_sol_val_calc_accs()
            .to_owned_copy();
        let reserves_addr = self
            .create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
            .ok_or(InfErr::NoValidPda)?;
        Ok((
            i as u32, // as-safety: i should not > u32::MAX
            lst_state,
            calc_accs,
            reserves_addr,
        ))
    }

    pub(crate) fn lst_vars_mut(&mut self, mint: &[u8; 32]) -> Result<LstVarsTup, InfErr> {
        let (i, lst_state) = try_find_lst_state(self.try_lst_state_list()?, mint)?;
        let calc_accs = self
            .try_get_or_init_lst_svc(&lst_state)?
            .as_sol_val_calc_accs()
            .to_owned_copy();
        let reserves_addr = self
            .create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
            .ok_or(InfErr::NoValidPda)?;
        Ok((
            i as u32, // as-safety: i should not > u32::MAX
            lst_state,
            calc_accs,
            reserves_addr,
        ))
    }
}

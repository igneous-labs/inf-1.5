//! General accounts update procedures.
//!
//! More specialized update procedures in respective folders
//! (e.g. update for trade is in update folder)

use inf1_core::inf1_ctl_core::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStatePacked},
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::lst_state::LstState,
};
use inf1_pp_ag_std::{
    update::{all::AccountsToUpdateAll, UpdatePricingProg},
    PricingProgAg,
};
use inf1_svc_ag_std::update::UpdateSvc;

// Re-exports
pub use inf1_svc_ag_std::update::{Account, UpdateErr, UpdateMap};

use crate::{
    err::InfErr,
    utils::{
        balance_from_token_acc_data, token_supply_from_mint_data,
        try_default_pricing_prog_from_program_id,
    },
    Inf, Reserves,
};

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
    #[inline]
    pub fn accounts_to_update_all(&self) -> impl Iterator<Item = [u8; 32]> + use<'_, F, C> {
        let lst_state_iter = self.lst_state_list().iter().map(|l| l.into_lst_state());
        [POOL_STATE_ID, LST_STATE_LIST_ID, self.pool.lp_token_mint]
            .into_iter()
            .chain(
                self.pricing.accounts_to_update_all(
                    lst_state_iter.clone().map(|LstState { mint, .. }| mint),
                ),
            )
            .chain(
                lst_state_iter
                    .filter_map(|lst_state| {
                        // ignore err here, some LSTs may not have their.
                        // sol val calc accounts fetched yet.
                        //
                        // update_all() should call `try_get_or_init_lst_svc_mut`
                        // which will make it no longer err for the next update cycle
                        self.accounts_to_update_for_lst(&lst_state).ok()
                    })
                    .flatten(),
            )
    }

    #[inline]
    pub fn update_all(&mut self, fetched: impl UpdateMap) -> Result<(), UpdateErr<InfErr>> {
        self.update_pool(&fetched)?;
        self.update_lst_state_list(&fetched)?;
        self.update_lp_token_supply(&fetched)?;

        // have to use raw defn of self.lst_state() instead of calling it here in order to avoid
        // borrowing entirety of self instead of just the lst_state_list_data field
        let mut all_lst_states = LstStatePackedList::of_acc_data(&self.lst_state_list_data)
            .unwrap()
            .0
            .iter()
            .map(|s| s.into_lst_state());

        self.pricing
            .update_all(
                all_lst_states.clone().map(|LstState { mint, .. }| mint),
                &fetched,
            )
            .map_err(|e| e.map_inner(InfErr::UpdatePp))?;

        all_lst_states.try_for_each(|lst_state| self.update_lst(&lst_state, &fetched))?;

        Ok(())
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
    /// Also replaces the pricing program data with fresh default if the pricing program was
    /// found to have changed.
    #[inline]
    pub fn update_pool(&mut self, fetched: impl UpdateMap) -> Result<(), UpdateErr<InfErr>> {
        let pool_state_acc = fetched.get_account_checked(&POOL_STATE_ID)?;

        let pool = PoolStatePacked::of_acc_data(pool_state_acc.data())
            .ok_or(UpdateErr::Inner(InfErr::AccDeser { pk: POOL_STATE_ID }))?
            .into_pool_state();

        if *self.pricing.0.ty().program_id() != pool.pricing_program {
            self.pricing = self
                .try_default_pricing_prog_from_program_id(&pool.pricing_program)
                .map_err(UpdateErr::Inner)?;
        }

        self.pool = pool;

        // TODO: maybe cleanup removed LSTs from self.lst_reserves and self.lst_calc?

        Ok(())
    }
}

impl<F, C> Inf<F, C> {
    #[inline]
    pub fn update_lst_state_list(
        &mut self,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        let lst_state_list_acc = fetched.get_account_checked(&LST_STATE_LIST_ID)?;
        if LstStatePackedList::of_acc_data(lst_state_list_acc.data()).is_none() {
            return Err(UpdateErr::Inner(InfErr::AccDeser {
                pk: inf1_core::inf1_ctl_core::keys::LST_STATE_LIST_ID,
            }));
        }
        self.lst_state_list_data = lst_state_list_acc.data().into();

        Ok(())
    }

    /// Must be called after [`Self::update_pool`]
    /// to use latest value of `pool.lp_token_mint`
    #[inline]
    pub fn update_lp_token_supply(
        &mut self,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        let lp_mint_acc = fetched.get_account_checked(&self.pool.lp_token_mint)?;
        let lp_token_supply = token_supply_from_mint_data(lp_mint_acc.data()).ok_or(
            UpdateErr::Inner(InfErr::AccDeser {
                pk: self.pool.lp_token_mint,
            }),
        )?;

        self.lp_token_supply = Some(lp_token_supply);

        Ok(())
    }
}

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    /// Must be called after [`Self::update_lst_state_list`]
    /// to use latest `LstState`s
    #[inline]
    pub fn update_lst(
        &mut self,
        lst_state: &LstState,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        // update calc
        let calc = self
            .try_get_or_init_lst_svc_mut(lst_state)
            .map_err(UpdateErr::Inner)?;
        calc.update_svc(&fetched)
            .map_err(|e| e.map_inner(InfErr::UpdateSvc))?;

        // update reserves
        let reserves_addr = self
            .create_pool_reserves_ata(&lst_state.mint, lst_state.pool_reserves_bump)
            .ok_or(UpdateErr::Inner(InfErr::NoValidPda))?;
        let token_acc = fetched.get_account_checked(&reserves_addr)?;
        let balance = balance_from_token_acc_data(token_acc.data())
            .ok_or(UpdateErr::Inner(InfErr::AccDeser { pk: reserves_addr }))?;
        self.lst_reserves
            .insert(lst_state.mint, Reserves { balance });

        Ok(())
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
    #[inline]
    pub fn try_default_pricing_prog_from_program_id(
        &self,
        program_id: &[u8; 32],
    ) -> Result<PricingProgAg<F, C>, InfErr> {
        try_default_pricing_prog_from_program_id(
            program_id,
            self.find_pda.clone(),
            self.create_pda.clone(),
        )
    }
}

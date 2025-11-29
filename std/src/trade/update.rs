use std::{
    array,
    iter::{once, Chain},
};

use inf1_core::{
    inf1_ctl_core::{
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        typedefs::lst_state::LstState,
    },
    inf1_pp_core::pair::Pair,
};
use inf1_pp_ag_std::update::{
    price_exact_in::AccountsToUpdatePriceExactIn, price_exact_out::AccountsToUpdatePriceExactOut,
    UpdatePricingProg,
};
use inf1_svc_ag_std::update::{AccountsToUpdateSvc, UpdateErr, UpdateMap};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    update::{UpdateLstPairPkIter, UpdateLstPkIter},
    utils::try_find_lst_state,
    Inf,
};

pub type TradeUpdatePkIter = Trade<UpdateSwapExactInPkIter, UpdateSwapExactOutPkIter>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    #[inline]
    pub fn accounts_to_update_trade_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeUpdatePkIter, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut(_) => self
                .accounts_to_update_swap_exact_out_mut(pair)
                .map(Trade::ExactOut),
            TradeLimitTy::ExactIn(_) => self
                .accounts_to_update_swap_exact_in_mut(pair)
                .map(Trade::ExactIn),
        }
    }

    #[inline]
    pub fn accounts_to_update_trade(
        &self,
        pair: &Pair<&[u8; 32]>,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeUpdatePkIter, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut(_) => self
                .accounts_to_update_swap_exact_out(pair)
                .map(Trade::ExactOut),
            TradeLimitTy::ExactIn(_) => self
                .accounts_to_update_swap_exact_in(pair)
                .map(Trade::ExactIn),
        }
    }
}

// TODO: need to special-case INF mint

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn accounts_to_update_lst(
        &self,
        LstState {
            mint,
            pool_reserves_bump,
            ..
        }: &LstState,
    ) -> Result<UpdateLstPkIter, InfErr> {
        let calc = self.try_get_lst_svc(mint)?;
        let reserves = self
            .create_pool_reserves_ata(mint, *pool_reserves_bump)
            .ok_or(InfErr::NoValidPda)?;
        Ok(calc.accounts_to_update_svc().chain(once(reserves)))
    }

    #[inline]
    pub fn accounts_to_update_lst_by_mint(
        &self,
        mint: &[u8; 32],
    ) -> Result<UpdateLstPkIter, InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.try_lst_state_list()?, mint)?;
        self.accounts_to_update_lst(&lst_state)
    }

    #[inline]
    pub fn accounts_to_update_lst_mut(
        &mut self,
        lst_state: &LstState,
    ) -> Result<UpdateLstPkIter, InfErr> {
        let calc_accs = self
            .try_get_or_init_lst_svc(lst_state)?
            .accounts_to_update_svc();
        let reserves = self
            .create_pool_reserves_ata(&lst_state.mint, lst_state.pool_reserves_bump)
            .ok_or(InfErr::NoValidPda)?;
        Ok(calc_accs.chain(once(reserves)))
    }

    #[inline]
    pub fn accounts_to_update_lst_by_mint_mut(
        &mut self,
        mint: &[u8; 32],
    ) -> Result<UpdateLstPkIter, InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.try_lst_state_list()?, mint)?;
        self.accounts_to_update_lst_mut(&lst_state)
    }
}

pub type UpdateSwapExactInPkIter = Chain<
    Chain<array::IntoIter<[u8; 32], 2>, UpdateLstPairPkIter>,
    inf1_pp_ag_std::update::price_exact_in::PkIter,
>;

pub type UpdateSwapExactOutPkIter = Chain<
    Chain<array::IntoIter<[u8; 32], 2>, UpdateLstPairPkIter>,
    inf1_pp_ag_std::update::price_exact_out::PkIter,
>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    #[inline]
    pub fn accounts_to_update_swap_exact_in(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_lst_pair(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_in_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_lst_pair_mut(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_lst_pair(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_out(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_lst_pair_mut(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_out(pair)))
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
    #[inline]
    pub fn update_trade(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        limit_ty: TradeLimitTy,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        match limit_ty {
            TradeLimitTy::ExactOut(_) => self.update_swap_exact_out(pair, fetched),
            TradeLimitTy::ExactIn(_) => self.update_swap_exact_in(pair, fetched),
        }
    }

    fn update_swap_common(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_pool(&fetched)?;
        self.update_lst_state_list(&fetched)?;
        self.update_lst_pair(pair, fetched)?;
        Ok(())
    }

    #[inline]
    pub fn update_swap_exact_in(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_swap_common(pair, &fetched)?;
        self.pricing
            .update_price_exact_in(pair, fetched)
            .map_err(|e| e.map_inner(InfErr::UpdatePp))?;
        Ok(())
    }

    #[inline]
    pub fn update_swap_exact_out(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_swap_common(pair, &fetched)?;
        self.pricing
            .update_price_exact_out(pair, fetched)
            .map_err(|e| e.map_inner(InfErr::UpdatePp))?;
        Ok(())
    }
}

use std::{
    array,
    iter::{once, Chain},
};

use inf1_core::{
    inf1_ctl_core::{
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        svc::InfCalc,
        typedefs::lst_state::LstState,
    },
    inf1_pp_core::pair::Pair,
};
use inf1_pp_ag_std::update::{
    price_exact_in::AccountsToUpdatePriceExactIn, price_exact_out::AccountsToUpdatePriceExactOut,
    UpdatePricingProg,
};
use inf1_svc_ag_std::{
    inf1_svc_inf_std::InfSvcStd,
    inf1_svc_lido_std::solido_legacy_core::SYSVAR_CLOCK,
    update::{AccountsToUpdateSvc, UpdateErr, UpdateMap},
    SvcAg, SvcAgStd,
};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    update::UpdateLstPkIter,
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

pub type UpdateSwapCommonPkIter =
    Chain<Chain<array::IntoIter<[u8; 32], 2>, UpdateLstPkIter>, UpdateLstPkIter>;

pub type UpdateSwapExactInPkIter =
    Chain<UpdateSwapCommonPkIter, inf1_pp_ag_std::update::price_exact_in::PkIter>;

pub type UpdateSwapExactOutPkIter =
    Chain<UpdateSwapCommonPkIter, inf1_pp_ag_std::update::price_exact_out::PkIter>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    #[inline]
    fn inf_svc_pks(&self) -> UpdateLstPkIter {
        // dont care abt calc quoting for getting pks
        let calc = self.inf_calc(0).unwrap_or(InfCalc::DEFAULT);
        SvcAgStd(SvcAg::Inf(InfSvcStd {
            calc,
            mint_addr: *self.pool.lp_token_mint(),
        }))
        .accounts_to_update_svc()
        // TODO: currently a happy coincidence that
        // InfSvc::accounts_to_update_svc doesnt have clock in accounts
        // and we need exactly clock so this calc requires 3 accounts for updates
        // like the other calcs. In the future we might want to change the iter type
        // away from UpdateLstPkIter
        .chain(once(SYSVAR_CLOCK))
    }

    #[inline]
    fn accounts_to_update_swap_common(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapCommonPkIter, InfErr> {
        let Pair { inp, out } = pair.try_map(|m| {
            if m == self.pool.lp_token_mint() {
                Ok(self.inf_svc_pks())
            } else {
                self.accounts_to_update_lst_by_mint(m)
            }
        })?;
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(inp)
            .chain(out))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_in(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        self.accounts_to_update_swap_common(pair)
            .map(|x| x.chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        self.accounts_to_update_swap_common(pair)
            .map(|x| x.chain(self.pricing.accounts_to_update_price_exact_out(pair)))
    }

    #[inline]
    fn accounts_to_update_swap_common_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapCommonPkIter, InfErr> {
        let Pair { inp, out } = pair.try_map(|m| {
            if m == self.pool.lp_token_mint() {
                Ok(self.inf_svc_pks())
            } else {
                self.accounts_to_update_lst_by_mint_mut(m)
            }
        })?;
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(inp)
            .chain(out))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_in_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        self.accounts_to_update_swap_common_mut(pair)
            .map(|x| x.chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        self.accounts_to_update_swap_common_mut(pair)
            .map(|x| x.chain(self.pricing.accounts_to_update_price_exact_out(pair)))
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

        pair.try_map(|mint| {
            if mint == self.pool.lp_token_mint() {
                self.update_lp_token_supply(&fetched)
            } else {
                let lst_state_list = self.try_lst_state_list().map_err(UpdateErr::Inner)?;
                let (_i, lst_state) =
                    try_find_lst_state(lst_state_list, mint).map_err(UpdateErr::Inner)?;
                self.update_lst(&lst_state, &fetched)
            }
        })?;

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

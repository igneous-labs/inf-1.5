use std::{
    array,
    iter::{once, Chain, Once},
};

use inf1_core::{
    inf1_ctl_core::keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    inf1_pp_core::pair::Pair,
};
use inf1_pp_ag_std::update::{
    mint_lp::AccountsToUpdateMintLp, price_exact_in::AccountsToUpdatePriceExactIn,
    price_exact_out::AccountsToUpdatePriceExactOut, redeem_lp::AccountsToUpdateRedeemLp,
    UpdatePricingProg,
};
use inf1_svc_ag_std::update::{AccountsToUpdateSvc, SvcPkIterAg, UpdateErr, UpdateMap};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    utils::{try_find_lst_state, try_map_pair},
    Inf,
};

pub type TradeUpdatePkIter = Trade<
    UpdateAddLiqPkIter,
    UpdateRemoveLiqPkIter,
    UpdateSwapExactInPkIter,
    UpdateSwapExactOutPkIter,
>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    #[inline]
    pub fn accounts_to_update_for_trade_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeUpdatePkIter, InfErr> {
        let lp_token_mint = self.pool.lp_token_mint;
        match limit_ty {
            TradeLimitTy::ExactOut => {
                // currently only swap is supported for ExactOut
                self.accounts_to_update_swap_exact_out_mut(pair)
                    .map(Trade::SwapExactOut)
            }
            TradeLimitTy::ExactIn => {
                if *pair.out == lp_token_mint {
                    self.accounts_to_update_add_liq_mut(pair.inp)
                        .map(Trade::AddLiquidity)
                } else if *pair.inp == lp_token_mint {
                    self.accounts_to_update_remove_liq_mut(pair.out)
                        .map(Trade::RemoveLiquidity)
                } else {
                    self.accounts_to_update_swap_exact_in_mut(pair)
                        .map(Trade::SwapExactIn)
                }
            }
        }
    }

    #[inline]
    pub fn accounts_to_update_for_trade(
        &self,
        pair: &Pair<&[u8; 32]>,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeUpdatePkIter, InfErr> {
        let lp_token_mint = self.pool.lp_token_mint;
        match limit_ty {
            TradeLimitTy::ExactOut => {
                // currently only swap is supported for ExactOut
                self.accounts_to_update_swap_exact_out(pair)
                    .map(Trade::SwapExactOut)
            }
            TradeLimitTy::ExactIn => {
                if *pair.out == lp_token_mint {
                    self.accounts_to_update_add_liq(pair.inp)
                        .map(Trade::AddLiquidity)
                } else if *pair.inp == lp_token_mint {
                    self.accounts_to_update_remove_liq(pair.out)
                        .map(Trade::RemoveLiquidity)
                } else {
                    self.accounts_to_update_swap_exact_in(pair)
                        .map(Trade::SwapExactIn)
                }
            }
        }
    }
}

pub type UpdateLstPkIter = Chain<SvcPkIterAg, Once<[u8; 32]>>;

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn accounts_to_update_for_lst(&self, mint: &[u8; 32]) -> Result<UpdateLstPkIter, InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
        let calc = self.try_get_lst_svc(mint)?;
        let reserves = self
            .create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
            .ok_or(InfErr::NoValidPda)?;
        Ok(calc.accounts_to_update_svc().chain(once(reserves)))
    }

    #[inline]
    pub fn accounts_to_update_for_lst_mut(
        &mut self,
        mint: &[u8; 32],
    ) -> Result<UpdateLstPkIter, InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
        let calc_accs = self
            .try_get_or_init_lst_svc_mut(&lst_state)?
            .accounts_to_update_svc();
        let reserves = self
            .create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
            .ok_or(InfErr::NoValidPda)?;
        Ok(calc_accs.chain(once(reserves)))
    }
}

pub type UpdateAddLiqPkIter = Chain<
    Chain<array::IntoIter<[u8; 32], 3>, UpdateLstPkIter>,
    inf1_pp_ag_std::update::mint_lp::PkIter,
>;

pub type UpdateRemoveLiqPkIter = Chain<
    Chain<array::IntoIter<[u8; 32], 3>, UpdateLstPkIter>,
    inf1_pp_ag_std::update::redeem_lp::PkIter,
>;

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn accounts_to_update_add_liq(
        &self,
        inp_mint: &[u8; 32],
    ) -> Result<UpdateAddLiqPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID, self.pool.lp_token_mint]
            .into_iter()
            .chain(self.accounts_to_update_for_lst(inp_mint)?)
            .chain(self.pricing.accounts_to_update_mint_lp()))
    }

    #[inline]
    pub fn accounts_to_update_add_liq_mut(
        &mut self,
        inp_mint: &[u8; 32],
    ) -> Result<UpdateAddLiqPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID, self.pool.lp_token_mint]
            .into_iter()
            .chain(self.accounts_to_update_for_lst_mut(inp_mint)?)
            .chain(self.pricing.accounts_to_update_mint_lp()))
    }

    #[inline]
    pub fn accounts_to_update_remove_liq(
        &self,
        out_mint: &[u8; 32],
    ) -> Result<UpdateRemoveLiqPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID, self.pool.lp_token_mint]
            .into_iter()
            .chain(self.accounts_to_update_for_lst(out_mint)?)
            .chain(self.pricing.accounts_to_update_redeem_lp()))
    }

    #[inline]
    pub fn accounts_to_update_remove_liq_mut(
        &mut self,
        out_mint: &[u8; 32],
    ) -> Result<UpdateRemoveLiqPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID, self.pool.lp_token_mint]
            .into_iter()
            .chain(self.accounts_to_update_for_lst_mut(out_mint)?)
            .chain(self.pricing.accounts_to_update_redeem_lp()))
    }
}

type UpdateSwapCommonPkIter = Chain<UpdateLstPkIter, UpdateLstPkIter>;

pub type UpdateSwapExactInPkIter = Chain<
    Chain<array::IntoIter<[u8; 32], 2>, UpdateSwapCommonPkIter>,
    inf1_pp_ag_std::update::price_exact_in::PkIter,
>;

pub type UpdateSwapExactOutPkIter = Chain<
    Chain<array::IntoIter<[u8; 32], 2>, UpdateSwapCommonPkIter>,
    inf1_pp_ag_std::update::price_exact_out::PkIter,
>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    fn accounts_to_update_swap_common(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapCommonPkIter, InfErr> {
        let Pair { inp, out } = try_map_pair(*pair, |mint| self.accounts_to_update_for_lst(mint))?;
        Ok(inp.chain(out))
    }

    fn accounts_to_update_swap_common_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapCommonPkIter, InfErr> {
        let Pair { inp, out } =
            try_map_pair(*pair, |mint| self.accounts_to_update_for_lst_mut(mint))?;
        Ok(inp.chain(out))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_in(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_swap_common(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_in_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_swap_common_mut(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_swap_common(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_out(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_swap_common_mut(pair)?)
            .chain(self.pricing.accounts_to_update_price_exact_out(pair)))
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
    #[inline]
    pub fn update_for_trade(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        limit_ty: TradeLimitTy,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        let lp_token_mint = self.pool.lp_token_mint;
        match limit_ty {
            TradeLimitTy::ExactOut => {
                // currently only swap is supported for ExactOut
                self.update_swap_exact_out(pair, fetched)
            }
            TradeLimitTy::ExactIn => {
                if *pair.out == lp_token_mint {
                    self.update_add_liq(pair.inp, fetched)
                } else if *pair.inp == lp_token_mint {
                    self.update_remove_liq(pair.out, fetched)
                } else {
                    self.update_swap_exact_in(pair, fetched)
                }
            }
        }
    }

    fn update_liq_common(
        &mut self,
        mint: &[u8; 32],
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_pool(&fetched)?;
        self.update_lst_state_list(&fetched)?;
        self.update_lp_token_supply(&fetched)?;
        self.update_lst(mint, fetched)?;
        Ok(())
    }

    #[inline]
    pub fn update_add_liq(
        &mut self,
        inp_mint: &[u8; 32],
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_liq_common(inp_mint, &fetched)?;
        self.pricing
            .update_mint_lp(fetched)
            .map_err(|e| e.map_inner(InfErr::UpdatePp))?;
        Ok(())
    }

    #[inline]
    pub fn update_remove_liq(
        &mut self,
        out_mint: &[u8; 32],
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_liq_common(out_mint, &fetched)?;
        self.pricing
            .update_redeem_lp(fetched)
            .map_err(|e| e.map_inner(InfErr::UpdatePp))?;
        Ok(())
    }

    fn update_swap_common(
        &mut self,
        Pair { inp, out }: &Pair<&[u8; 32]>,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_pool(&fetched)?;
        self.update_lst_state_list(&fetched)?;
        self.update_lst(inp, &fetched)?;
        self.update_lst(out, fetched)?;
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

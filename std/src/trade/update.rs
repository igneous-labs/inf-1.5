use std::{array, iter::Chain};

use inf1_core::{
    inf1_ctl_core::{
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        typedefs::lst_state::LstState,
    },
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
    utils::{try_find_lst_state, try_map_pair},
    Inf,
};

type UpdateLiqCommonPkIter = Chain<array::IntoIter<[u8; 32], 2>, array::IntoIter<[u8; 32], 2>>;

pub type UpdateAddLiqPkIter =
    Chain<Chain<UpdateLiqCommonPkIter, SvcPkIterAg>, inf1_pp_ag_std::update::mint_lp::PkIter>;

pub type UpdateRemoveLiqPkIter =
    Chain<Chain<UpdateLiqCommonPkIter, SvcPkIterAg>, inf1_pp_ag_std::update::redeem_lp::PkIter>;

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    fn accounts_to_update_liq_common(
        &self,
        lst_mint: &[u8; 32],
    ) -> Result<(LstState, UpdateLiqCommonPkIter), InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.lst_state_list(), lst_mint)?;
        Ok((
            lst_state,
            [POOL_STATE_ID, LST_STATE_LIST_ID].into_iter().chain([
                self.pool.lp_token_mint,
                self.create_pool_reserves_ata(lst_mint, lst_state.pool_reserves_bump)
                    .ok_or(InfErr::NoValidPda)?,
            ]),
        ))
    }

    #[inline]
    pub fn accounts_to_update_add_liq(
        &self,
        inp_mint: &[u8; 32],
    ) -> Result<UpdateAddLiqPkIter, InfErr> {
        let (lst_state, common) = self.accounts_to_update_liq_common(inp_mint)?;
        let calc = self.try_get_lst_svc(&lst_state.mint)?;
        Ok(common
            .chain(calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_mint_lp()))
    }

    #[inline]
    pub fn accounts_to_update_add_liq_mut(
        &mut self,
        inp_mint: &[u8; 32],
    ) -> Result<UpdateAddLiqPkIter, InfErr> {
        let (lst_state, common) = self.accounts_to_update_liq_common(inp_mint)?;
        let calc = self.try_get_or_init_lst_svc_mut(&lst_state)?;
        Ok(common
            .chain(calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_mint_lp()))
    }

    #[inline]
    pub fn accounts_to_update_remove_liq(
        &self,
        out_mint: &[u8; 32],
    ) -> Result<UpdateRemoveLiqPkIter, InfErr> {
        let (lst_state, common) = self.accounts_to_update_liq_common(out_mint)?;
        let calc = self.try_get_lst_svc(&lst_state.mint)?;
        Ok(common
            .chain(calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_redeem_lp()))
    }

    #[inline]
    pub fn accounts_to_update_remove_liq_mut(
        &mut self,
        out_mint: &[u8; 32],
    ) -> Result<UpdateRemoveLiqPkIter, InfErr> {
        let (lst_state, common) = self.accounts_to_update_liq_common(out_mint)?;
        let calc = self.try_get_or_init_lst_svc_mut(&lst_state)?;
        Ok(common
            .chain(calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_redeem_lp()))
    }
}

type UpdateSwapCommonPkIter = Chain<array::IntoIter<[u8; 32], 2>, array::IntoIter<[u8; 32], 2>>;

pub type UpdateSwapExactInPkIter = Chain<
    Chain<Chain<UpdateSwapCommonPkIter, SvcPkIterAg>, SvcPkIterAg>,
    inf1_pp_ag_std::update::price_exact_in::PkIter,
>;

pub type UpdateSwapExactOutPkIter = Chain<
    Chain<Chain<UpdateSwapCommonPkIter, SvcPkIterAg>, SvcPkIterAg>,
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
    ) -> Result<(Pair<LstState>, UpdateSwapCommonPkIter), InfErr> {
        let Pair {
            inp: inp_lst_state,
            out: out_lst_state,
        } = try_map_pair(*pair, |mint| {
            let (_, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
            Ok(lst_state)
        })?;
        let Pair {
            inp: inp_reserves,
            out: out_reserves,
        } = try_map_pair(
            Pair {
                inp: (pair.inp, &inp_lst_state),
                out: (pair.out, &out_lst_state),
            },
            |(mint, lst_state)| {
                self.create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
                    .ok_or(InfErr::NoValidPda)
            },
        )?;
        Ok((
            Pair {
                inp: inp_lst_state,
                out: out_lst_state,
            },
            [POOL_STATE_ID, LST_STATE_LIST_ID]
                .into_iter()
                .chain([inp_reserves, out_reserves]),
        ))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_in(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        let (_, common) = self.accounts_to_update_swap_common(pair)?;
        let Pair {
            inp: inp_calc,
            out: out_calc,
        } = try_map_pair(*pair, |mint| self.try_get_lst_svc(mint))?;
        Ok(common
            .chain(inp_calc.accounts_to_update_svc())
            .chain(out_calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_in_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        let (Pair { inp, out }, common) = self.accounts_to_update_swap_common(pair)?;
        let inp_calc = *self.try_get_or_init_lst_svc_mut(&inp)?;
        let out_calc = self.try_get_or_init_lst_svc_mut(&out)?;
        Ok(common
            .chain(inp_calc.accounts_to_update_svc())
            .chain(out_calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_price_exact_in(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out(
        &self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        let (_, common) = self.accounts_to_update_swap_common(pair)?;
        let Pair {
            inp: inp_calc,
            out: out_calc,
        } = try_map_pair(*pair, |mint| self.try_get_lst_svc(mint))?;
        Ok(common
            .chain(inp_calc.accounts_to_update_svc())
            .chain(out_calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_price_exact_out(pair)))
    }

    #[inline]
    pub fn accounts_to_update_swap_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateSwapExactInPkIter, InfErr> {
        let (Pair { inp, out }, common) = self.accounts_to_update_swap_common(pair)?;
        let inp_calc = *self.try_get_or_init_lst_svc_mut(&inp)?;
        let out_calc = self.try_get_or_init_lst_svc_mut(&out)?;
        Ok(common
            .chain(inp_calc.accounts_to_update_svc())
            .chain(out_calc.accounts_to_update_svc())
            .chain(self.pricing.accounts_to_update_price_exact_out(pair)))
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
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

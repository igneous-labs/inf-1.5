use std::{array, iter::Chain};

use inf1_core::inf1_ctl_core::keys::{LST_STATE_LIST_ID, POOL_STATE_ID};
use inf1_pp_ag_std::update::all::Pair;
use inf1_svc_ag_std::update::{UpdateErr, UpdateMap};

use crate::{err::InfErr, update::UpdateLstPairPkIter, Inf};

pub type UpdateRebalancePkIter = Chain<array::IntoIter<[u8; 32], 2>, UpdateLstPairPkIter>;

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn accounts_to_update_rebalance_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
    ) -> Result<UpdateRebalancePkIter, InfErr> {
        Ok([POOL_STATE_ID, LST_STATE_LIST_ID]
            .into_iter()
            .chain(self.accounts_to_update_lst_pair_mut(pair)?))
    }
}

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)> + Clone,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]> + Clone,
    > Inf<F, C>
{
    #[inline]
    pub fn update_rebalance(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        fetched: impl UpdateMap,
    ) -> Result<(), UpdateErr<InfErr>> {
        self.update_pool(&fetched)?;
        self.update_lst_state_list(&fetched)?;
        self.update_lst_pair(pair, fetched)?;
        Ok(())
    }
}

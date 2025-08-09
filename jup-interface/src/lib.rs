use std::sync::{atomic::AtomicU64, Arc};

use anyhow::Result;
use inf1_std::{
    err::InfErr,
    inf1_ctl_core::{
        accounts::lst_state_list::LstStatePackedList,
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        typedefs::lst_state::LstState,
    },
    inf1_pp_ag_std::update::{all::AccountsToUpdateAll, UpdatePricingProg},
    inf1_svc_ag_std::{
        inf1_svc_lido_core::solido_legacy_core::SYSVAR_CLOCK, update::UpdateSvc, SvcAg,
    },
    update::UpdateErr,
    InfStd,
};
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, KeyedAccount, Quote, QuoteParams, SwapAndAccountMetas, SwapParams,
};
use solana_pubkey::Pubkey;

use crate::update::AccountMapRef;

// mod consts;
// mod pda;
mod update;

// Note on Clock hax:
// Because `Clock` is a special-case account, and because it's only used
// by Lido and Spl SolValCalcs to check current epoch to filter out unexecutable quoting rn:
// - we exclude it from all update accounts
// - update procedures use the `_no_clock()` variants that dont
//   update clock data and hence dont rely on clock acc being in AccountMap
// - `current_epoch=0` on all the SolValCalc structs so that quoting will never
//   fail due to the underlying stake pool not being updated for the epoch
// - we only check for underlying stake pool not being updated for the epoch
//   at the end of quoting procedure to determine whether to return the quote or err

#[derive(Debug, Clone)]
pub struct Inf {
    pub inner: InfStd,
    pub current_epoch: Arc<AtomicU64>,
}

impl Amm for Inf {
    /// The `keyed_account` should be the `LST_STATE_LIST`, **NOT** `POOL_STATE`.
    fn from_keyed_account(_keyed_account: &KeyedAccount, _amm_context: &AmmContext) -> Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn label(&self) -> String {
        todo!()
    }

    fn program_id(&self) -> Pubkey {
        todo!()
    }

    fn key(&self) -> Pubkey {
        todo!()
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        todo!()
    }

    /// Note: does not dedup
    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        let lst_state_iter = self
            .inner
            .try_lst_state_list()
            .unwrap_or_default() // TODO: should this panic instead if LstStateList format unexpectedly changed?
            .iter()
            .map(|l| l.into_lst_state());
        [
            POOL_STATE_ID,
            LST_STATE_LIST_ID,
            self.inner.pool.lp_token_mint,
        ]
        .into_iter()
        .chain(
            self.inner
                .pricing
                .accounts_to_update_all(lst_state_iter.clone().map(|LstState { mint, .. }| mint)),
        )
        .chain(
            lst_state_iter
                .filter_map(|lst_state| {
                    // ignore err here, some LSTs may not have their.
                    // sol val calc accounts fetched yet.
                    //
                    // update() should call `try_get_or_init_lst_svc_mut`
                    // which will make it no longer err for the next update cycle
                    self.inner
                        .accounts_to_update_for_lst(&lst_state)
                        .ok()
                        .map(|iter| iter.filter(|pk| *pk != SYSVAR_CLOCK))
                })
                .flatten(),
        )
        .map(Pubkey::new_from_array)
        .collect()
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let fetched = AccountMapRef(account_map);
        self.inner.update_pool(fetched)?;
        self.inner.update_lst_state_list(fetched)?;
        self.inner.update_lp_token_supply(fetched)?;

        let InfStd {
            lst_state_list_data,
            pricing,
            lst_calcs,
            spl_lsts,
            lst_reserves,
            create_pda,
            ..
        } = &mut self.inner;

        let mut all_lst_states = LstStatePackedList::of_acc_data(lst_state_list_data)
            .ok_or(InfErr::AccDeser {
                pk: LST_STATE_LIST_ID,
            })?
            .0
            .iter()
            .map(|s| s.into_lst_state());

        pricing.update_all(
            all_lst_states.clone().map(|LstState { mint, .. }| mint),
            fetched,
        )?;

        all_lst_states.try_for_each(|lst_state| {
            InfStd::update_lst_reserves(lst_reserves, create_pda as &_, &lst_state, fetched)?;

            let calc = InfStd::try_get_or_init_lst_svc_static(lst_calcs, spl_lsts, &lst_state)
                .map_err(UpdateErr::Inner)?;
            match calc.0 {
                // omit clock for these variants
                SvcAg::Lido(mut c) => c
                    .update_svc_no_clock(fetched)
                    .map_err(|e| e.map_inner(SvcAg::Lido).map_inner(InfErr::UpdateSvc)),
                SvcAg::SanctumSpl(mut c) => c
                    .update_svc_no_clock(fetched)
                    .map_err(|e| e.map_inner(SvcAg::SanctumSpl).map_inner(InfErr::UpdateSvc)),
                SvcAg::SanctumSplMulti(mut c) => c.update_svc_no_clock(fetched).map_err(|e| {
                    e.map_inner(SvcAg::SanctumSplMulti)
                        .map_inner(InfErr::UpdateSvc)
                }),
                SvcAg::Spl(mut c) => c
                    .update_svc_no_clock(fetched)
                    .map_err(|e| e.map_inner(SvcAg::Spl).map_inner(InfErr::UpdateSvc)),

                // following variants unaffected by clock
                SvcAg::Marinade(mut c) => c
                    .update_svc(fetched)
                    .map_err(|e| e.map_inner(SvcAg::Marinade).map_inner(InfErr::UpdateSvc)),
                SvcAg::Wsol(mut c) => c
                    .update_svc(fetched)
                    .map_err(|e| e.map_inner(SvcAg::Wsol).map_inner(InfErr::UpdateSvc)),
            }
        })?;

        Ok(())
    }

    fn quote(&self, _quote_params: &QuoteParams) -> Result<Quote> {
        todo!()
    }

    fn get_swap_and_account_metas(&self, _swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        todo!()
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        todo!()
    }

    fn has_dynamic_accounts(&self) -> bool {
        true
    }

    /// TODO: this is not true for AddLiquidity and RemoveLiquidity
    fn supports_exact_out(&self) -> bool {
        true
    }

    fn program_dependencies(&self) -> Vec<(Pubkey, String)> {
        todo!()
    }
}

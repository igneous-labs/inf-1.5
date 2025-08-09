use anyhow::Result;
use inf1_std::InfStd;
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, KeyedAccount, Quote, QuoteParams, SwapAndAccountMetas, SwapParams,
};
use solana_pubkey::Pubkey;

use crate::update::AccountMapRef;

// mod consts;
// mod pda;
mod update;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Inf(pub InfStd);

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
        self.0
            .accounts_to_update_all()
            .map(Pubkey::new_from_array)
            .collect()
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        self.0.update_all(AccountMapRef(account_map))?;
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

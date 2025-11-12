use std::{
    collections::HashMap,
    iter::once,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use ::sanctum_lst_list::{PoolInfo, SanctumLst};
use anyhow::{anyhow, Context, Result};
use inf1_std::{
    err::InfErr,
    inf1_ctl_core::{
        accounts::lst_state_list::LstStatePackedList,
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        typedefs::lst_state::LstState,
    },
    inf1_pp_ag_std::{
        inf1_pp_flatfee_core,
        update::{all::AccountsToUpdateAll, UpdatePricingProg},
    },
    inf1_pp_core::pair::Pair,
    inf1_svc_ag_std::{
        inf1_svc_lido_core::{self, calc::LidoCalcErr, solido_legacy_core::SYSVAR_CLOCK},
        inf1_svc_marinade_core,
        inf1_svc_spl_core::{self, calc::SplCalcErr},
        inf1_svc_wsol_core,
        update::UpdateSvc,
        SvcAg,
    },
    instructions::swap::{
        exact_in::{
            swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer, swap_exact_in_ix_keys_owned,
        },
        exact_out::{
            swap_exact_out_ix_is_signer, swap_exact_out_ix_is_writer, swap_exact_out_ix_keys_owned,
        },
    },
    quote::swap::err::SwapQuoteErr,
    trade::{instruction::TradeIxArgs, Trade, TradeLimitTy},
    update::UpdateErr,
    InfStd,
};
use jupiter_amm_interface::{
    single_program_amm, AccountMap, Amm, AmmContext, KeyedAccount, Quote, QuoteParams,
    SingleProgramAmm, Swap, SwapAndAccountMetas, SwapMode, SwapParams,
};
use rust_decimal::Decimal;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

use crate::{
    clock::is_epoch_affected_lst_mint,
    consts::{DEFAULT_MAINNET_POOL, LABEL},
    err::FmtErr,
    pda::{create_raw_pda, find_pda},
    sanctum_lst_list::load_sanctum_lst_list,
    update::AccountMapRef,
};

#[allow(deprecated)]
use inf1_std::instructions::liquidity::{
    add::{add_liquidity_ix_is_signer, add_liquidity_ix_is_writer, add_liquidity_ix_keys_owned},
    remove::{
        remove_liquidity_ix_is_signer, remove_liquidity_ix_is_writer,
        remove_liquidity_ix_keys_owned,
    },
};

pub mod clock;
pub mod consts;
pub mod err;
pub mod update;

mod pda;
mod sanctum_lst_list;

pub const INF_PROGRAM_ID: Pubkey = Pubkey::new_from_array(inf1_std::inf1_ctl_core::ID);
pub const INF_LST_LIST_ID: Pubkey = Pubkey::new_from_array(LST_STATE_LIST_ID);

// Note on Clock hax:
// Because `Clock` is a special-case account, and because it's only used
// by Lido and Spl SolValCalcs to check current epoch to filter out unexecutable quoting rn:
// - we exclude it from all update accounts
// - update procedures use the `_no_clock()` variants that dont
//   update clock data and hence dont rely on clock acc being in AccountMap
// - `current_epoch=0` on all the SolValCalc structs so that quoting will never
//   fail due to the underlying stake pool not being updated for the epoch
// - we only check for underlying stake pool not being updated for the epoch
//   during the quoting procedure to determine whether to return err

#[derive(Debug, Clone)]
pub struct InfAmm {
    pub inner: InfStd,
    pub current_epoch: Arc<AtomicU64>,
}
single_program_amm!(InfAmm, INF_PROGRAM_ID, LABEL);

fn build_spl_lsts() -> HashMap<[u8; 32], [u8; 32]> {
    load_sanctum_lst_list()
        .into_iter()
        .filter_map(|SanctumLst { mint, pool, .. }| {
            let stake_pool_address = match pool {
                PoolInfo::Lido => return None,
                PoolInfo::Marinade => return None,
                PoolInfo::ReservePool => return None,
                PoolInfo::SanctumSpl(spl_pool_accounts) => spl_pool_accounts.pool.to_bytes(),
                PoolInfo::Spl(spl_pool_accounts) => spl_pool_accounts.pool.to_bytes(),
                PoolInfo::SPool(_) => return None,
                PoolInfo::SanctumSplMulti(spl_pool_accounts) => spl_pool_accounts.pool.to_bytes(),
            };
            Some((mint.to_bytes(), stake_pool_address))
        })
        .collect()
}

impl Amm for InfAmm {
    /// The `keyed_account` should be the `LST_STATE_LIST`, **NOT** `POOL_STATE`.
    fn from_keyed_account(keyed_account: &KeyedAccount, amm_context: &AmmContext) -> Result<Self>
    where
        Self: Sized,
    {
        if *keyed_account.key.as_array() != LST_STATE_LIST_ID {
            return Err(anyhow!("Incorrect LST state list keyed_account"));
        }

        let mut res = Self {
            inner: InfStd::new(
                DEFAULT_MAINNET_POOL,
                keyed_account.account.data.clone().into_boxed_slice(),
                None,
                None,
                Default::default(),
                Default::default(),
                build_spl_lsts(),
                find_pda,
                create_raw_pda,
            )
            .map_err(FmtErr)?,
            current_epoch: amm_context.clock_ref.epoch.clone(),
        };

        // need to initialize sol val calc data for all LSTs on the list
        // so that first update doesnt fail with InfErr::MissingSvcData

        let lst_state_list = LstStatePackedList::of_acc_data(&keyed_account.account.data)
            .context("LstStatePackedList::of_acc_data failed")?;
        lst_state_list
            .0
            .iter()
            .try_for_each(
                |s| match res.inner.try_get_or_init_lst_svc(&s.into_lst_state()) {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        if matches!(error, InfErr::MissingSplData { .. }) {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    }
                },
            )
            .map_err(FmtErr)?;

        Ok(res)
    }

    fn label(&self) -> String {
        LABEL.to_owned()
    }

    fn program_id(&self) -> Pubkey {
        inf1_std::inf1_ctl_core::ID.into()
    }

    /// S Pools are 1 per program, so just use program ID as key
    fn key(&self) -> Pubkey {
        INF_LST_LIST_ID
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        let lst_state_list = self.inner.try_lst_state_list().unwrap_or_default();
        lst_state_list
            .iter()
            .map(|s| s.into_lst_state().mint.into())
            .chain(once(self.inner.pool.lp_token_mint.into()))
            .collect()
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
                        .accounts_to_update_lst(&lst_state)
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
        self.inner.update_pool(fetched).map_err(FmtErr)?;
        self.inner.update_lst_state_list(fetched).map_err(FmtErr)?;
        self.inner.update_lp_token_supply(fetched).map_err(FmtErr)?;

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
            .ok_or(FmtErr(InfErr::AccDeser {
                pk: LST_STATE_LIST_ID,
            }))?
            .0
            .iter()
            .map(|s| s.into_lst_state());

        pricing.update_all(
            all_lst_states.clone().map(|LstState { mint, .. }| mint),
            fetched,
        )?;

        all_lst_states
            .try_for_each(|lst_state| {
                InfStd::update_lst_reserves(lst_reserves, create_pda as &_, &lst_state, fetched)?;

                let calc =
                    match InfStd::try_get_or_init_lst_svc_static(lst_calcs, spl_lsts, &lst_state) {
                        Ok(calc) => calc,
                        Err(error) => {
                            // Do not cause an error when we don't have the necessary spl data for a LST
                            if matches!(error, InfErr::MissingSplData { .. }) {
                                lst_calcs.remove(&lst_state.mint);
                                return Ok(());
                            } else {
                                return Err(UpdateErr::Inner(error));
                            }
                        }
                    };

                match &mut calc.0 {
                    // omit clock for these variants
                    SvcAg::Lido(c) => c
                        .update_svc_no_clock(fetched)
                        .map_err(|e| e.map_inner(SvcAg::Lido).map_inner(InfErr::UpdateSvc)),
                    SvcAg::SanctumSpl(c) => c
                        .update_svc_no_clock(fetched)
                        .map_err(|e| e.map_inner(SvcAg::SanctumSpl).map_inner(InfErr::UpdateSvc)),
                    SvcAg::SanctumSplMulti(c) => c.update_svc_no_clock(fetched).map_err(|e| {
                        e.map_inner(SvcAg::SanctumSplMulti)
                            .map_inner(InfErr::UpdateSvc)
                    }),
                    SvcAg::Spl(c) => c
                        .update_svc_no_clock(fetched)
                        .map_err(|e| e.map_inner(SvcAg::Spl).map_inner(InfErr::UpdateSvc)),

                    // following variants unaffected by clock
                    SvcAg::Marinade(c) => c
                        .update_svc(fetched)
                        .map_err(|e| e.map_inner(SvcAg::Marinade).map_inner(InfErr::UpdateSvc)),
                    SvcAg::Wsol(c) => c
                        .update_svc(fetched)
                        .map_err(|e| e.map_inner(SvcAg::Wsol).map_inner(InfErr::UpdateSvc)),
                }
            })
            .map_err(FmtErr)?;

        Ok(())
    }

    fn quote(
        &self,
        QuoteParams {
            amount,
            input_mint,
            output_mint,
            swap_mode,
            ..
        }: &QuoteParams,
    ) -> Result<Quote> {
        // clock special-case handling:
        // early return err if any of the mints are
        // epoch affected and epoch conditions dont hold
        for mint in [input_mint, output_mint] {
            let mint = mint.as_array();
            if !is_epoch_affected_lst_mint(mint) {
                continue;
            }
            // since INF is not clock affected, we dont need to
            // worry about try_get_lst_svc() failing for it.
            // In future vers, INF will also have its own sol val calc anyway.
            match self
                .inner
                .try_get_lst_svc(mint)
                .map_err(FmtErr)?
                .as_sol_val_calc()
            {
                // kinda sloppy, but if NotUpdated err encountered, just return it under
                // SwapQuoteErr::InpCalc instead of determining what kind of swap and
                // what position the affected mint was in
                Some(c) => match c {
                    SvcAg::Marinade(_) | SvcAg::Wsol(_) => continue,
                    SvcAg::Lido(c) => {
                        if c.exchange_rate.computed_in_epoch
                            < self.current_epoch.load(Ordering::Relaxed)
                        {
                            return Err(FmtErr(InfErr::SwapQuote(SwapQuoteErr::InpCalc(
                                SvcAg::Lido(LidoCalcErr::NotUpdated),
                            )))
                            .into());
                        }
                    }
                    SvcAg::SanctumSpl(c) | SvcAg::SanctumSplMulti(c) | SvcAg::Spl(c) => {
                        if c.last_update_epoch < self.current_epoch.load(Ordering::Relaxed) {
                            return Err(FmtErr(InfErr::SwapQuote(SwapQuoteErr::InpCalc(
                                SvcAg::Spl(SplCalcErr::NotUpdated),
                            )))
                            .into());
                        }
                    }
                },
                None => return Err(FmtErr(InfErr::MissingSvcData { mint: *mint }).into()),
            }
        }

        match self
            .inner
            .quote_trade(
                &Pair {
                    inp: input_mint.as_array(),
                    out: output_mint.as_array(),
                },
                *amount,
                swap_mode_to_trade_limit_ty(*swap_mode),
            )
            .map_err(FmtErr)?
        {
            #[allow(deprecated)]
            Trade::AddLiquidity(q) => to_jup_quote(q.fee_mint(), q.0),
            #[allow(deprecated)]
            Trade::RemoveLiquidity(q) => to_jup_quote(q.fee_mint(), q.0),
            Trade::SwapExactIn(q) => to_jup_quote(q.fee_mint(), q.0),
            Trade::SwapExactOut(q) => to_jup_quote(q.fee_mint(), q.0),
        }
    }

    fn get_swap_and_account_metas(
        &self,
        SwapParams {
            swap_mode,
            in_amount,
            out_amount,
            source_mint,
            destination_mint,
            source_token_account,
            destination_token_account,
            token_transfer_authority,
            ..
        }: &SwapParams,
    ) -> Result<SwapAndAccountMetas> {
        let limit_ty = swap_mode_to_trade_limit_ty(*swap_mode);
        let (amt, limit) = match limit_ty {
            TradeLimitTy::ExactIn => (in_amount, out_amount),
            TradeLimitTy::ExactOut => (out_amount, in_amount),
        };
        let args = TradeIxArgs {
            amt: *amt,
            limit: *limit,
            mints: &Pair {
                inp: source_mint.as_array(),
                out: destination_mint.as_array(),
            },
            signer: token_transfer_authority.as_array(),
            token_accs: &Pair {
                inp: source_token_account.as_array(),
                out: destination_token_account.as_array(),
            },
        };
        let ix = self.inner.trade_ix(&args, limit_ty).map_err(FmtErr)?;
        let mut account_metas = vec![AccountMeta::new_readonly(Self::PROGRAM_ID, false)];
        Ok(match ix {
            Trade::AddLiquidity(ix) => {
                let a = ix.to_full();
                #[allow(deprecated)]
                account_metas.extend(keys_signer_writable_to_metas(
                    add_liquidity_ix_keys_owned(&ix.accs).seq(),
                    add_liquidity_ix_is_signer(&ix.accs).seq(),
                    add_liquidity_ix_is_writer(&ix.accs).seq(),
                ));
                SwapAndAccountMetas {
                    swap: Swap::SanctumSAddLiquidity {
                        lst_value_calc_accs: a.lst_value_calc_accs,
                        lst_index: a.lst_index,
                    },

                    account_metas,
                }
            }
            Trade::RemoveLiquidity(ix) => {
                let a = ix.to_full();
                #[allow(deprecated)]
                account_metas.extend(keys_signer_writable_to_metas(
                    remove_liquidity_ix_keys_owned(&ix.accs).seq(),
                    remove_liquidity_ix_is_signer(&ix.accs).seq(),
                    remove_liquidity_ix_is_writer(&ix.accs).seq(),
                ));
                SwapAndAccountMetas {
                    swap: Swap::SanctumSRemoveLiquidity {
                        lst_value_calc_accs: a.lst_value_calc_accs,
                        lst_index: a.lst_index,
                    },

                    account_metas,
                }
            }
            Trade::SwapExactIn(ix) => {
                let a = ix.to_full();
                #[allow(deprecated)]
                account_metas.extend(keys_signer_writable_to_metas(
                    swap_exact_in_ix_keys_owned(&ix.accs).seq(),
                    swap_exact_in_ix_is_signer(&ix.accs).seq(),
                    swap_exact_in_ix_is_writer(&ix.accs).seq(),
                ));
                SwapAndAccountMetas {
                    swap: Swap::SanctumS {
                        src_lst_value_calc_accs: a.inp_lst_value_calc_accs,
                        dst_lst_value_calc_accs: a.out_lst_value_calc_accs,
                        src_lst_index: a.inp_lst_index,
                        dst_lst_index: a.out_lst_index,
                    },
                    account_metas,
                }
            }
            Trade::SwapExactOut(ix) => {
                let a = ix.to_full();
                #[allow(deprecated)]
                account_metas.extend(keys_signer_writable_to_metas(
                    swap_exact_out_ix_keys_owned(&ix.accs).seq(),
                    swap_exact_out_ix_is_signer(&ix.accs).seq(),
                    swap_exact_out_ix_is_writer(&ix.accs).seq(),
                ));
                SwapAndAccountMetas {
                    swap: Swap::SanctumS {
                        src_lst_value_calc_accs: a.inp_lst_value_calc_accs,
                        dst_lst_value_calc_accs: a.out_lst_value_calc_accs,
                        src_lst_index: a.inp_lst_index,
                        dst_lst_index: a.out_lst_index,
                    },
                    account_metas,
                }
            }
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }

    fn has_dynamic_accounts(&self) -> bool {
        true
    }

    fn supports_exact_out(&self) -> bool {
        false
    }

    fn program_dependencies(&self) -> Vec<(Pubkey, String)> {
        PROGRAM_DEPENDENCIES
            .into_iter()
            .map(|(program_id, label)| (program_id.into(), label.into()))
            .collect()
    }

    fn get_accounts_len(&self) -> usize {
        32
    }
}

pub const PROGRAM_DEPENDENCIES: [([u8; 32], &str); 12] = [
    // SPL
    (inf1_svc_spl_core::keys::spl::POOL_PROG_ID, "spl_stake_pool"),
    (inf1_svc_spl_core::keys::spl::ID, "spl_calculator"),
    // Sanctum SPL
    (
        inf1_svc_spl_core::keys::sanctum_spl::POOL_PROG_ID,
        "sanctum_spl_stake_pool",
    ),
    (
        inf1_svc_spl_core::keys::sanctum_spl::ID,
        "sanctum_spl_calculator",
    ),
    // Sanctum SPL Multi
    (
        inf1_svc_spl_core::keys::sanctum_spl_multi::POOL_PROG_ID,
        "sanctum_spl_multi_stake_pool",
    ),
    (
        inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
        "sanctum_spl_multi_calculator",
    ),
    // marinade
    (inf1_svc_marinade_core::keys::POOL_PROG_ID, "marinade"),
    (inf1_svc_marinade_core::ID, "marinade_calculator"),
    // lido
    (inf1_svc_lido_core::keys::POOL_PROG_ID, "lido"),
    (inf1_svc_lido_core::ID, "lido_calculator"),
    // wSOL
    (inf1_svc_wsol_core::ID, "wsol_calculator"),
    // pricing program
    (inf1_pp_flatfee_core::ID, "flat_fee_pricing_program"),
];

#[inline]
pub const fn swap_mode_to_trade_limit_ty(sm: SwapMode) -> TradeLimitTy {
    match sm {
        SwapMode::ExactIn => TradeLimitTy::ExactIn,
        SwapMode::ExactOut => TradeLimitTy::ExactOut,
    }
}

#[inline]
pub fn to_jup_quote(
    fee_mint: &[u8; 32],
    inf1_std::quote::Quote {
        inp: in_amount,
        out: out_amount,
        lp_fee,
        protocol_fee,
        inp_mint,
        out_mint: _,
    }: inf1_std::quote::Quote,
) -> Result<Quote, anyhow::Error> {
    let fee_amount = lp_fee.saturating_add(protocol_fee);
    let fee_pct_f64 = {
        let denom = if *fee_mint == inp_mint {
            in_amount
        } else {
            out_amount.saturating_add(fee_amount)
        };
        (fee_amount as f64) / (denom as f64)
    };
    let fee_pct = Decimal::from_f64_retain(fee_pct_f64).ok_or_else(|| anyhow!("Decimal err"))?;
    Ok(Quote {
        in_amount,
        out_amount,
        fee_amount,
        fee_mint: Pubkey::new_from_array(*fee_mint),
        fee_pct,
    })
}

pub fn keys_signer_writable_to_metas<'a>(
    keys: impl Iterator<Item = &'a [u8; 32]>,
    signer: impl Iterator<Item = &'a bool>,
    writable: impl Iterator<Item = &'a bool>,
) -> Vec<AccountMeta> {
    keys.zip(signer)
        .zip(writable)
        .map(|((key, _signer), writable)| AccountMeta {
            pubkey: Pubkey::new_from_array(*key),
            is_signer: false, // The signer is elevated by the jupiter instruction, otherwise uses shared accounts and elevated internally before CPI
            is_writable: *writable,
        })
        .collect()
}

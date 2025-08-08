#![allow(deprecated)]

use inf1_core::{
    inf1_ctl_core::{
        instructions::{
            liquidity::{
                add::{AddLiquidityIxPreAccs, NewAddLiquidityIxPreAccsBuilder},
                remove::{NewRemoveLiquidityIxPreAccsBuilder, RemoveLiquidityIxPreAccs},
            },
            swap::{IxPreAccs as SwapIxPreAccs, NewIxPreAccsBuilder as NewSwapIxPreAccsBuilder},
        },
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        typedefs::lst_state::LstState,
    },
    inf1_pp_core::{
        pair::Pair,
        traits::{
            collection::{PriceExactInAccsCol, PriceExactOutAccsCol},
            deprecated::{PriceLpTokensToMintAccsCol, PriceLpTokensToRedeemAccsCol},
        },
    },
    instructions::{
        liquidity::{
            add::{AddLiquidityIxAccs, AddLiquidityIxArgs},
            remove::{RemoveLiquidityIxAccs, RemoveLiquidityIxArgs},
        },
        swap::{
            exact_in::{SwapExactInIxAccs, SwapExactInIxArgs},
            exact_out::{SwapExactOutIxAccs, SwapExactOutIxArgs},
        },
    },
};
use inf1_pp_ag_std::instructions::{
    PriceExactInAccsAg, PriceExactOutAccsAg, PriceLpTokensToMintAccsAg, PriceLpTokensToRedeemAccsAg,
};
use inf1_svc_ag_std::{
    inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::TOKEN_PROGRAM,
    instructions::SvcCalcAccsAg,
};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    utils::{try_find_lst_state, try_map_pair},
    Inf,
};

pub type AddLiquidityIxArgsStd = AddLiquidityIxArgs<
    [u8; 32],
    AddLiquidityIxPreAccs<[u8; 32]>,
    SvcCalcAccsAg,
    PriceLpTokensToMintAccsAg,
>;

pub type RemoveLiquidityIxArgsStd = RemoveLiquidityIxArgs<
    [u8; 32],
    RemoveLiquidityIxPreAccs<[u8; 32]>,
    SvcCalcAccsAg,
    PriceLpTokensToRedeemAccsAg,
>;

pub type SwapExactInIxArgsStd = SwapExactInIxArgs<
    [u8; 32],
    SwapIxPreAccs<[u8; 32]>,
    SvcCalcAccsAg,
    SvcCalcAccsAg,
    PriceExactInAccsAg,
>;

pub type SwapExactOutIxArgsStd = SwapExactOutIxArgs<
    [u8; 32],
    SwapIxPreAccs<[u8; 32]>,
    SvcCalcAccsAg,
    SvcCalcAccsAg,
    PriceExactOutAccsAg,
>;

pub type TradeIxArgsStd = Trade<
    AddLiquidityIxArgsStd,
    RemoveLiquidityIxArgsStd,
    SwapExactInIxArgsStd,
    SwapExactOutIxArgsStd,
>;

#[derive(Debug, Clone, Copy)]
pub struct TradeIxArgs<'a> {
    pub amt: u64,
    pub limit: u64,

    /// `inp` is ignored for remove liquidity.
    /// `out` is ignored for add liquidity,
    pub mints: &'a Pair<&'a [u8; 32]>,

    pub signer: &'a [u8; 32],
    pub token_accs: &'a Pair<&'a [u8; 32]>,
}

/// (lst_index, lst_state, lst_calc_accs, lst_reserves_addr)
type LstVarsTup = (u32, LstState, SvcCalcAccsAg, [u8; 32]);

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    fn lst_vars(&self, mint: &[u8; 32]) -> Result<LstVarsTup, InfErr> {
        let (i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
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

    fn lst_vars_mut(&mut self, mint: &[u8; 32]) -> Result<LstVarsTup, InfErr> {
        let (i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
        let calc_accs = self
            .try_get_or_init_lst_svc_mut(&lst_state)?
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

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    #[inline]
    pub fn trade_ix_mut(
        &mut self,
        args: &TradeIxArgs,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeIxArgsStd, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut => {
                // currently only swap is supported for ExactOut
                self.swap_exact_out_ix_mut(args).map(Trade::SwapExactOut)
            }
            TradeLimitTy::ExactIn => {
                let lp_token_mint = self.pool.lp_token_mint;
                if *args.mints.out == lp_token_mint {
                    self.add_liq_ix_mut(args).map(Trade::AddLiquidity)
                } else if *args.mints.inp == lp_token_mint {
                    self.remove_liq_ix_mut(args).map(Trade::RemoveLiquidity)
                } else {
                    self.swap_exact_in_ix_mut(args).map(Trade::SwapExactIn)
                }
            }
        }
    }

    #[inline]
    pub fn trade_ix(
        &self,
        args: &TradeIxArgs,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeIxArgsStd, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut => {
                // currently only swap is supported for ExactOut
                self.swap_exact_out_ix(args).map(Trade::SwapExactOut)
            }
            TradeLimitTy::ExactIn => {
                let lp_token_mint = self.pool.lp_token_mint;
                if *args.mints.out == lp_token_mint {
                    self.add_liq_ix(args).map(Trade::AddLiquidity)
                } else if *args.mints.inp == lp_token_mint {
                    self.remove_liq_ix(args).map(Trade::RemoveLiquidity)
                } else {
                    self.swap_exact_in_ix(args).map(Trade::SwapExactIn)
                }
            }
        }
    }

    // AddLiquidity

    fn add_liq_ix_pre_accs(
        &self,
        TradeIxArgs {
            mints: Pair { inp: inp_mint, .. },
            signer,
            token_accs:
                Pair {
                    inp: inp_token_acc,
                    out: out_token_acc,
                },
            ..
        }: &TradeIxArgs,
        pool_reserves: [u8; 32],
        protocol_fee_accumulator: [u8; 32],
    ) -> AddLiquidityIxPreAccs<[u8; 32]> {
        NewAddLiquidityIxPreAccsBuilder::start()
            .with_pool_reserves(pool_reserves)
            .with_protocol_fee_accumulator(protocol_fee_accumulator)
            .with_signer(**signer)
            .with_lst_acc(**inp_token_acc)
            .with_lp_acc(**out_token_acc)
            .with_lst_mint(**inp_mint)
            .with_lp_token_mint(self.pool.lp_token_mint)
            .with_lst_token_program(TOKEN_PROGRAM)
            .with_lp_token_program(TOKEN_PROGRAM)
            .with_lst_state_list(LST_STATE_LIST_ID)
            .with_pool_state(POOL_STATE_ID)
            .build()
    }

    fn add_liq_ix_common(
        &self,
        args: &TradeIxArgs,
        (lst_index, lst_state, lst_calc, pool_reserves): LstVarsTup,
    ) -> Result<AddLiquidityIxArgsStd, InfErr> {
        let pfa = self
            .create_protocol_fee_accumulator_ata(
                args.mints.inp,
                lst_state.protocol_fee_accumulator_bump,
            )
            .ok_or(InfErr::NoValidPda)?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_mint_accs_for(args.mints.inp)
            .map_err(InfErr::PricingProg)?;
        let accs = AddLiquidityIxAccs {
            ix_prefix: self.add_liq_ix_pre_accs(args, pool_reserves, pfa),
            lst_calc_prog: lst_state.sol_value_calculator,
            lst_calc,
            pricing_prog: self.pool.pricing_program,
            pricing,
        };
        Ok(AddLiquidityIxArgs {
            lst_index,
            amount: args.amt,
            min_out: args.limit,
            accs,
        })
    }

    #[inline]
    pub fn add_liq_ix_mut(&mut self, args: &TradeIxArgs) -> Result<AddLiquidityIxArgsStd, InfErr> {
        let tup = self.lst_vars_mut(args.mints.inp)?;
        self.add_liq_ix_common(args, tup)
    }

    #[inline]
    pub fn add_liq_ix(&self, args: &TradeIxArgs) -> Result<AddLiquidityIxArgsStd, InfErr> {
        let tup = self.lst_vars(args.mints.inp)?;
        self.add_liq_ix_common(args, tup)
    }

    // RemoveLiquidity

    fn remove_liq_ix_pre_accs(
        &self,
        TradeIxArgs {
            mints: Pair { out: out_mint, .. },
            signer,
            token_accs:
                Pair {
                    inp: inp_token_acc,
                    out: out_token_acc,
                },
            ..
        }: &TradeIxArgs,
        pool_reserves: [u8; 32],
        protocol_fee_accumulator: [u8; 32],
    ) -> AddLiquidityIxPreAccs<[u8; 32]> {
        NewRemoveLiquidityIxPreAccsBuilder::start()
            .with_pool_reserves(pool_reserves)
            .with_protocol_fee_accumulator(protocol_fee_accumulator)
            .with_signer(**signer)
            .with_lst_acc(**out_token_acc)
            .with_lp_acc(**inp_token_acc)
            .with_lst_mint(**out_mint)
            .with_lp_token_mint(self.pool.lp_token_mint)
            .with_lst_token_program(TOKEN_PROGRAM)
            .with_lp_token_program(TOKEN_PROGRAM)
            .with_lst_state_list(LST_STATE_LIST_ID)
            .with_pool_state(POOL_STATE_ID)
            .build()
    }

    fn remove_liq_ix_common(
        &self,
        args: &TradeIxArgs,
        (lst_index, lst_state, lst_calc, pool_reserves): LstVarsTup,
    ) -> Result<RemoveLiquidityIxArgsStd, InfErr> {
        let protocol_fee_accumulator = self
            .create_protocol_fee_accumulator_ata(
                args.mints.out,
                lst_state.protocol_fee_accumulator_bump,
            )
            .ok_or(InfErr::NoValidPda)?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_redeem_accs_for(args.mints.out)
            .map_err(InfErr::PricingProg)?;
        let accs = RemoveLiquidityIxAccs {
            ix_prefix: self.remove_liq_ix_pre_accs(args, pool_reserves, protocol_fee_accumulator),
            lst_calc_prog: lst_state.sol_value_calculator,
            lst_calc,
            pricing_prog: self.pool.pricing_program,
            pricing,
        };

        Ok(RemoveLiquidityIxArgs {
            lst_index,
            amount: args.amt,
            min_out: args.limit,
            accs,
        })
    }

    #[inline]
    pub fn remove_liq_ix(&self, args: &TradeIxArgs) -> Result<RemoveLiquidityIxArgsStd, InfErr> {
        let tup = self.lst_vars(args.mints.out)?;
        self.remove_liq_ix_common(args, tup)
    }

    #[inline]
    pub fn remove_liq_ix_mut(
        &mut self,
        args: &TradeIxArgs,
    ) -> Result<RemoveLiquidityIxArgsStd, InfErr> {
        let tup = self.lst_vars_mut(args.mints.out)?;
        self.remove_liq_ix_common(args, tup)
    }

    // swap common

    fn swap_ix_pre_accs(
        &self,
        TradeIxArgs {
            mints:
                Pair {
                    out: out_mint,
                    inp: inp_mint,
                },
            signer,
            token_accs:
                Pair {
                    inp: inp_token_acc,
                    out: out_token_acc,
                },
            ..
        }: &TradeIxArgs,
        Pair {
            inp: (_, _, _, inp_reserves),
            out: (_, _, _, out_reserves),
        }: &Pair<LstVarsTup>,
        out_protocol_fee_accumulator: [u8; 32],
    ) -> SwapIxPreAccs<[u8; 32]> {
        NewSwapIxPreAccsBuilder::start()
            .with_inp_pool_reserves(*inp_reserves)
            .with_out_pool_reserves(*out_reserves)
            .with_protocol_fee_accumulator(out_protocol_fee_accumulator)
            .with_signer(**signer)
            .with_inp_lst_acc(**inp_token_acc)
            .with_out_lst_acc(**out_token_acc)
            .with_inp_lst_mint(**inp_mint)
            .with_out_lst_mint(**out_mint)
            .with_inp_lst_token_program(TOKEN_PROGRAM)
            .with_out_lst_token_program(TOKEN_PROGRAM)
            .with_lst_state_list(LST_STATE_LIST_ID)
            .with_pool_state(POOL_STATE_ID)
            .build()
    }

    // SwapExactIn

    fn swap_exact_in_ix_common(
        &self,
        args: &TradeIxArgs,
        vars: Pair<LstVarsTup>,
    ) -> Result<SwapExactInIxArgsStd, InfErr> {
        let Pair {
            inp: (inp_lst_index, inp_lst_state, inp_calc, _),
            out: (out_lst_index, out_lst_state, out_calc, _),
        } = vars;
        let out_protocol_fee_accumulator = self
            .create_protocol_fee_accumulator_ata(
                args.mints.out,
                vars.out.1.protocol_fee_accumulator_bump,
            )
            .ok_or(InfErr::NoValidPda)?;
        let pricing = self
            .pricing
            .price_exact_in_accs_for(args.mints)
            .map_err(InfErr::PricingProg)?;
        let accs = SwapExactInIxAccs {
            ix_prefix: self.swap_ix_pre_accs(args, &vars, out_protocol_fee_accumulator),
            pricing_prog: self.pool.pricing_program,
            pricing,
            inp_calc_prog: inp_lst_state.sol_value_calculator,
            inp_calc,
            out_calc_prog: out_lst_state.sol_value_calculator,
            out_calc,
        };
        Ok(SwapExactInIxArgs {
            amount: args.amt,
            limit: args.limit,
            accs,
            inp_lst_index,
            out_lst_index,
        })
    }

    #[inline]
    pub fn swap_exact_in_ix(&self, args: &TradeIxArgs) -> Result<SwapExactInIxArgsStd, InfErr> {
        let vars = try_map_pair(*args.mints, |mint| self.lst_vars(mint))?;
        self.swap_exact_in_ix_common(args, vars)
    }

    #[inline]
    pub fn swap_exact_in_ix_mut(
        &mut self,
        args: &TradeIxArgs,
    ) -> Result<SwapExactInIxArgsStd, InfErr> {
        let vars = try_map_pair(*args.mints, |mint| self.lst_vars_mut(mint))?;
        self.swap_exact_in_ix_common(args, vars)
    }

    // SwapExactOut

    fn swap_exact_out_ix_common(
        &self,
        args: &TradeIxArgs,
        vars: Pair<LstVarsTup>,
    ) -> Result<SwapExactInIxArgsStd, InfErr> {
        let Pair {
            inp: (inp_lst_index, inp_lst_state, inp_calc, _),
            out: (out_lst_index, out_lst_state, out_calc, _),
        } = vars;
        let out_protocol_fee_accumulator = self
            .create_protocol_fee_accumulator_ata(
                args.mints.out,
                vars.out.1.protocol_fee_accumulator_bump,
            )
            .ok_or(InfErr::NoValidPda)?;
        let pricing = self
            .pricing
            .price_exact_out_accs_for(args.mints)
            .map_err(InfErr::PricingProg)?;
        let accs = SwapExactOutIxAccs {
            ix_prefix: self.swap_ix_pre_accs(args, &vars, out_protocol_fee_accumulator),
            pricing_prog: self.pool.pricing_program,
            pricing,
            inp_calc_prog: inp_lst_state.sol_value_calculator,
            inp_calc,
            out_calc_prog: out_lst_state.sol_value_calculator,
            out_calc,
        };
        Ok(SwapExactOutIxArgs {
            amount: args.amt,
            limit: args.limit,
            accs,
            inp_lst_index,
            out_lst_index,
        })
    }

    #[inline]
    pub fn swap_exact_out_ix(&self, args: &TradeIxArgs) -> Result<SwapExactOutIxArgsStd, InfErr> {
        let vars = try_map_pair(*args.mints, |mint| self.lst_vars(mint))?;
        self.swap_exact_out_ix_common(args, vars)
    }

    #[inline]
    pub fn swap_exact_out_ix_mut(
        &mut self,
        args: &TradeIxArgs,
    ) -> Result<SwapExactOutIxArgsStd, InfErr> {
        let vars = try_map_pair(*args.mints, |mint| self.lst_vars_mut(mint))?;
        self.swap_exact_out_ix_common(args, vars)
    }
}

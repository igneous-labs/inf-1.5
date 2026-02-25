use inf1_core::{
    inf1_ctl_core::{
        instructions::swap::v2::{
            IxPreAccs as SwapV2IxPreAccs, NewIxPreAccsBuilder as NewSwapV2IxPreAccsBuilder,
        },
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
        svc::InfDummyCalcAccs,
    },
    inf1_pp_core::{
        pair::Pair,
        traits::collection::{PriceExactInAccsCol, PriceExactOutAccsCol},
    },
    instructions::swap::{IxAccs as SwapIxAccs, IxArgs as SwapIxArgs},
};
use inf1_pp_ag_std::{
    inf1_pp_flatfee_std::instructions::pricing::price::FlatFeePriceAccs,
    inf1_pp_flatslab_std::instructions::pricing::FlatSlabPpAccs, PricingAg,
};
use inf1_svc_ag_std::{
    inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::TOKEN_PROGRAM,
    instructions::SvcCalcAccsAg,
};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    Inf, LstVarsTup,
};

pub type SwapIxArgsStd = SwapIxArgs<
    [u8; 32],
    SwapV2IxPreAccs<[u8; 32]>,
    SvcCalcAccsAg,
    SvcCalcAccsAg,
    PricingAg<FlatFeePriceAccs, FlatSlabPpAccs>,
>;

pub type TradeIxArgsStd = Trade<SwapIxArgsStd, SwapIxArgsStd>;

#[derive(Debug, Clone, Copy)]
pub struct TradeIxArgs<'a> {
    pub amt: u64,
    pub limit: u64,

    /// `inp` is LP token for remove liquidity.
    /// `out` is LP token for add liquidity,
    pub mints: &'a Pair<&'a [u8; 32]>,

    pub signer: &'a [u8; 32],
    pub token_accs: &'a Pair<&'a [u8; 32]>,
}

enum TokenVars {
    Lst(LstVarsTup),
    Lp(
        /// LP token mint (to be used as pool inp/out reserves account)
        [u8; 32],
    ),
}

impl TokenVars {
    #[inline]
    pub const fn lst_index(&self) -> u32 {
        match self {
            Self::Lst((i, ..)) => *i,
            Self::Lp(_) => u32::MAX,
        }
    }

    #[inline]
    pub const fn svc_calc_accs_ag(&self) -> SvcCalcAccsAg {
        match self {
            Self::Lst((_, _, calc, ..)) => *calc,
            Self::Lp(_) => SvcCalcAccsAg::Inf(InfDummyCalcAccs),
        }
    }

    #[inline]
    pub const fn svc_prog_id(&self) -> &[u8; 32] {
        match self {
            Self::Lst((_, s, ..)) => &s.sol_value_calculator,
            Self::Lp(_) => &inf1_core::inf1_ctl_core::ID,
        }
    }

    #[inline]
    pub const fn reserves_addr(&self) -> &[u8; 32] {
        match self {
            Self::Lst((_, _, _, a)) | Self::Lp(a) => a,
        }
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
            TradeLimitTy::ExactOut(_) => self.swap_exact_out_ix_mut(args).map(Trade::ExactOut),
            TradeLimitTy::ExactIn(_) => self.swap_exact_in_ix_mut(args).map(Trade::ExactIn),
        }
    }

    #[inline]
    pub fn trade_ix(
        &self,
        args: &TradeIxArgs,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeIxArgsStd, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut(_) => self.swap_exact_out_ix(args).map(Trade::ExactOut),
            TradeLimitTy::ExactIn(_) => self.swap_exact_in_ix(args).map(Trade::ExactIn),
        }
    }

    // swap common

    #[inline]
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
            inp: inp_vars,
            out: out_vars,
        }: &Pair<TokenVars>,
    ) -> SwapV2IxPreAccs<[u8; 32]> {
        NewSwapV2IxPreAccsBuilder::start()
            .with_inp_pool_reserves(*inp_vars.reserves_addr())
            .with_out_pool_reserves(*out_vars.reserves_addr())
            .with_signer(**signer)
            .with_inp_acc(**inp_token_acc)
            .with_out_acc(**out_token_acc)
            .with_inp_mint(**inp_mint)
            .with_out_mint(**out_mint)
            // TODO: token-22 support
            .with_inp_token_program(TOKEN_PROGRAM)
            .with_out_token_program(TOKEN_PROGRAM)
            .with_lst_state_list(LST_STATE_LIST_ID)
            .with_pool_state(POOL_STATE_ID)
            .build()
    }

    // SwapExactIn

    #[inline]
    fn swap_exact_in_ix_common(
        &self,
        args: &TradeIxArgs,
        vars: &Pair<TokenVars>,
    ) -> Result<SwapIxArgsStd, InfErr> {
        let Pair {
            inp: inp_vars,
            out: out_vars,
        } = vars;
        let pricing = self
            .pricing
            .price_exact_in_accs_for(args.mints)
            .map_err(InfErr::PricingProg)?;
        let accs = SwapIxAccs {
            ix_prefix: self.swap_ix_pre_accs(args, vars),
            pricing_prog: *self.pool.pricing_program(),
            pricing,
            inp_calc_prog: *inp_vars.svc_prog_id(),
            inp_calc: inp_vars.svc_calc_accs_ag(),
            out_calc_prog: *out_vars.svc_prog_id(),
            out_calc: out_vars.svc_calc_accs_ag(),
        };
        Ok(SwapIxArgs {
            amount: args.amt,
            limit: args.limit,
            accs,
            inp_lst_index: inp_vars.lst_index(),
            out_lst_index: out_vars.lst_index(),
        })
    }

    #[inline]
    pub fn swap_exact_in_ix(&self, args: &TradeIxArgs) -> Result<SwapIxArgsStd, InfErr> {
        let vars = args.mints.try_map(|mint| {
            if mint == self.pool.lp_token_mint() {
                Ok(TokenVars::Lp(*self.pool.lp_token_mint()))
            } else {
                self.lst_vars(mint).map(TokenVars::Lst)
            }
        })?;
        self.swap_exact_in_ix_common(args, &vars)
    }

    #[inline]
    pub fn swap_exact_in_ix_mut(&mut self, args: &TradeIxArgs) -> Result<SwapIxArgsStd, InfErr> {
        let vars = args.mints.try_map(|mint| {
            if mint == self.pool.lp_token_mint() {
                Ok(TokenVars::Lp(*self.pool.lp_token_mint()))
            } else {
                self.lst_vars_mut(mint).map(TokenVars::Lst)
            }
        })?;
        self.swap_exact_in_ix_common(args, &vars)
    }

    // SwapExactOut

    #[inline]
    fn swap_exact_out_ix_common(
        &self,
        args: &TradeIxArgs,
        vars: &Pair<TokenVars>,
    ) -> Result<SwapIxArgsStd, InfErr> {
        let Pair {
            inp: inp_vars,
            out: out_vars,
        } = vars;
        let pricing = self
            .pricing
            .price_exact_out_accs_for(args.mints)
            .map_err(InfErr::PricingProg)?;
        let accs = SwapIxAccs {
            ix_prefix: self.swap_ix_pre_accs(args, vars),
            pricing_prog: *self.pool.pricing_program(),
            pricing,
            inp_calc_prog: *inp_vars.svc_prog_id(),
            inp_calc: inp_vars.svc_calc_accs_ag(),
            out_calc_prog: *out_vars.svc_prog_id(),
            out_calc: out_vars.svc_calc_accs_ag(),
        };
        Ok(SwapIxArgs {
            amount: args.amt,
            limit: args.limit,
            accs,
            inp_lst_index: inp_vars.lst_index(),
            out_lst_index: out_vars.lst_index(),
        })
    }

    #[inline]
    pub fn swap_exact_out_ix(&self, args: &TradeIxArgs) -> Result<SwapIxArgsStd, InfErr> {
        let vars = args.mints.try_map(|mint| {
            if mint == self.pool.lp_token_mint() {
                Ok(TokenVars::Lp(*self.pool.lp_token_mint()))
            } else {
                self.lst_vars(mint).map(TokenVars::Lst)
            }
        })?;
        self.swap_exact_out_ix_common(args, &vars)
    }

    #[inline]
    pub fn swap_exact_out_ix_mut(&mut self, args: &TradeIxArgs) -> Result<SwapIxArgsStd, InfErr> {
        let vars = args.mints.try_map(|mint| {
            if mint == self.pool.lp_token_mint() {
                Ok(TokenVars::Lp(*self.pool.lp_token_mint()))
            } else {
                self.lst_vars_mut(mint).map(TokenVars::Lst)
            }
        })?;
        self.swap_exact_out_ix_common(args, &vars)
    }
}

use inf1_core::{
    inf1_ctl_core::{
        instructions::swap::v1::{
            IxPreAccs as SwapIxPreAccs, NewIxPreAccsBuilder as NewSwapIxPreAccsBuilder,
        },
        keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    },
    inf1_pp_core::{
        pair::Pair,
        traits::collection::{PriceExactInAccsCol, PriceExactOutAccsCol},
    },
    instructions::swap::v1::{
        exact_in::{SwapExactInIxAccs, SwapExactInIxArgs},
        exact_out::{SwapExactOutIxAccs, SwapExactOutIxArgs},
    },
};
use inf1_pp_ag_std::instructions::{PriceExactInAccsAg, PriceExactOutAccsAg};
use inf1_svc_ag_std::{
    inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::TOKEN_PROGRAM,
    instructions::SvcCalcAccsAg,
};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    Inf, LstVarsTup,
};

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

pub type TradeIxArgsStd = Trade<SwapExactInIxArgsStd, SwapExactOutIxArgsStd>;

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
        let vars = args.mints.try_map(|mint| self.lst_vars(mint))?;
        self.swap_exact_in_ix_common(args, vars)
    }

    #[inline]
    pub fn swap_exact_in_ix_mut(
        &mut self,
        args: &TradeIxArgs,
    ) -> Result<SwapExactInIxArgsStd, InfErr> {
        let vars = args.mints.try_map(|mint| self.lst_vars_mut(mint))?;
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
        let vars = args.mints.try_map(|mint| self.lst_vars(mint))?;
        self.swap_exact_out_ix_common(args, vars)
    }

    #[inline]
    pub fn swap_exact_out_ix_mut(
        &mut self,
        args: &TradeIxArgs,
    ) -> Result<SwapExactOutIxArgsStd, InfErr> {
        let vars = args.mints.try_map(|mint| self.lst_vars_mut(mint))?;
        self.swap_exact_out_ix_common(args, vars)
    }
}

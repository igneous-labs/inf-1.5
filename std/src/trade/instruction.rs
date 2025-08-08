#![allow(deprecated)]

use inf1_core::{
    inf1_ctl_core::{
        instructions::{
            liquidity::{
                add::{AddLiquidityIxPreAccs, NewAddLiquidityIxPreAccsBuilder},
                remove::{NewRemoveLiquidityIxPreAccsBuilder, RemoveLiquidityIxPreAccs},
            },
            swap::{
                exact_in::{NewSwapExactInIxPreAccsBuilder, SwapExactInIxPreAccs},
                exact_out::{NewSwapExactOutIxPreAccsBuilder, SwapExactOutIxPreAccs},
            },
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

use crate::{err::InfErr, utils::try_find_lst_state, Inf};

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
    SwapExactInIxPreAccs<[u8; 32]>,
    SvcCalcAccsAg,
    SvcCalcAccsAg,
    PriceExactInAccsAg,
>;

pub type SwapExactOutIxArgsStd = SwapExactOutIxArgs<
    [u8; 32],
    SwapExactOutIxPreAccs<[u8; 32]>,
    SvcCalcAccsAg,
    SvcCalcAccsAg,
    PriceExactOutAccsAg,
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

type LiqCommonTup = (u32, [u8; 32], [u8; 32], SvcCalcAccsAg, LstState);

type SwapCommonTup = (u32, LstState, SvcCalcAccsAg, [u8; 32]);

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > Inf<F, C>
{
    /// Returns (idx, pool_reserves, protocol_fee_accum, calc_accs, lst_state)
    fn liq_ix_common(&mut self, mint: &[u8; 32]) -> Result<LiqCommonTup, InfErr> {
        let (i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
        let calc = self
            .try_get_or_init_lst_svc_mut(&lst_state)?
            .as_sol_val_calc_accs()
            .to_owned_copy();
        let reserves = self
            .create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
            .ok_or(InfErr::NoValidPda)?;
        let pfa = self
            .create_protocol_fee_accumulator_ata(mint, lst_state.protocol_fee_accumulator_bump)
            .ok_or(InfErr::NoValidPda)?;
        Ok((
            i as u32, // as-safety: i should not > u32::MAX
            reserves, pfa, calc, lst_state,
        ))
    }

    #[inline]
    pub fn add_liq_ix(
        &mut self,
        TradeIxArgs {
            amt,
            limit,
            mints: Pair { inp: inp_mint, .. },
            signer,
            token_accs:
                Pair {
                    inp: inp_token_acc,
                    out: out_token_acc,
                },
        }: &TradeIxArgs,
    ) -> Result<AddLiquidityIxArgsStd, InfErr> {
        let (lst_index, pool_reserves, pfa, lst_calc, lst_state) = self.liq_ix_common(inp_mint)?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_mint_accs_for(inp_mint)
            .map_err(InfErr::PricingProg)?;

        let accs = AddLiquidityIxAccs {
            ix_prefix: NewAddLiquidityIxPreAccsBuilder::start()
                .with_pool_reserves(pool_reserves)
                .with_protocol_fee_accumulator(pfa)
                .with_signer(**signer)
                .with_lst_acc(**inp_token_acc)
                .with_lp_acc(**out_token_acc)
                .with_lst_mint(**inp_mint)
                .with_lp_token_mint(self.pool.lp_token_mint)
                .with_lst_token_program(TOKEN_PROGRAM)
                .with_lp_token_program(TOKEN_PROGRAM)
                .with_lst_state_list(LST_STATE_LIST_ID)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            lst_calc_prog: lst_state.sol_value_calculator,
            lst_calc,
            pricing_prog: self.pool.pricing_program,
            pricing,
        };

        Ok(AddLiquidityIxArgs {
            lst_index,
            amount: *amt,
            min_out: *limit,
            accs,
        })
    }

    #[inline]
    pub fn remove_liq_ix(
        &mut self,
        TradeIxArgs {
            amt,
            limit,
            mints: Pair { out: out_mint, .. },
            signer,
            token_accs:
                Pair {
                    inp: inp_token_acc,
                    out: out_token_acc,
                },
        }: &TradeIxArgs,
    ) -> Result<RemoveLiquidityIxArgsStd, InfErr> {
        let (lst_index, pool_reserves, pfa, lst_calc, lst_state) = self.liq_ix_common(out_mint)?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_redeem_accs_for(out_mint)
            .map_err(InfErr::PricingProg)?;

        let accs = RemoveLiquidityIxAccs {
            ix_prefix: NewRemoveLiquidityIxPreAccsBuilder::start()
                .with_pool_reserves(pool_reserves)
                .with_protocol_fee_accumulator(pfa)
                .with_signer(**signer)
                .with_lst_acc(**out_token_acc)
                .with_lp_acc(**inp_token_acc)
                .with_lst_mint(**out_mint)
                .with_lp_token_mint(self.pool.lp_token_mint)
                .with_lst_token_program(TOKEN_PROGRAM)
                .with_lp_token_program(TOKEN_PROGRAM)
                .with_lst_state_list(LST_STATE_LIST_ID)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            lst_calc_prog: lst_state.sol_value_calculator,
            lst_calc,
            pricing_prog: self.pool.pricing_program,
            pricing,
        };

        Ok(RemoveLiquidityIxArgs {
            lst_index,
            amount: *amt,
            min_out: *limit,
            accs,
        })
    }

    /// Returns `(inp_vars, out_vars, out_pf_accum))`
    #[allow(clippy::type_complexity)]
    fn swap_ix_common(
        &mut self,
        Pair { inp, out }: &Pair<&[u8; 32]>,
    ) -> Result<(SwapCommonTup, SwapCommonTup, [u8; 32]), InfErr> {
        let [inp_tup, out_tup] = [inp, out].map(|mint| {
            let (i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
            let calc = self
                .try_get_or_init_lst_svc_mut(&lst_state)?
                .as_sol_val_calc_accs()
                .to_owned_copy();
            let reserves = self
                .create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
                .ok_or(InfErr::NoValidPda)?;
            Ok::<_, InfErr>((
                i as u32, // as-safety: i should not > u32::MAX
                lst_state, calc, reserves,
            ))
        });
        let inp_tup = inp_tup?;
        let out_tup = out_tup?;

        let out_pfa = self
            .create_protocol_fee_accumulator_ata(out, out_tup.1.protocol_fee_accumulator_bump)
            .ok_or(InfErr::NoValidPda)?;

        Ok((inp_tup, out_tup, out_pfa))
    }

    #[inline]
    pub fn swap_exact_in_ix(
        &mut self,
        TradeIxArgs {
            amt,
            limit,
            mints,
            signer,
            token_accs:
                Pair {
                    inp: inp_token_acc,
                    out: out_token_acc,
                },
        }: &TradeIxArgs,
    ) -> Result<SwapExactInIxArgsStd, InfErr> {
        let Pair {
            out: out_mint,
            inp: inp_mint,
        } = mints;
        let (
            (inp_lst_index, inp_lst_state, inp_calc, inp_reserves),
            (out_lst_index, out_lst_state, out_calc, out_reserves),
            pfa,
        ) = self.swap_ix_common(mints)?;
        let pricing = self
            .pricing
            .price_exact_in_accs_for(mints)
            .map_err(InfErr::PricingProg)?;

        let accs = SwapExactInIxAccs {
            ix_prefix: NewSwapExactInIxPreAccsBuilder::start()
                .with_inp_pool_reserves(inp_reserves)
                .with_out_pool_reserves(out_reserves)
                .with_protocol_fee_accumulator(pfa)
                .with_signer(**signer)
                .with_inp_lst_acc(**inp_token_acc)
                .with_out_lst_acc(**out_token_acc)
                .with_inp_lst_mint(**inp_mint)
                .with_out_lst_mint(**out_mint)
                .with_inp_lst_token_program(TOKEN_PROGRAM)
                .with_out_lst_token_program(TOKEN_PROGRAM)
                .with_lst_state_list(LST_STATE_LIST_ID)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            pricing_prog: self.pool.pricing_program,
            pricing,
            inp_calc_prog: inp_lst_state.sol_value_calculator,
            inp_calc,
            out_calc_prog: out_lst_state.sol_value_calculator,
            out_calc,
        };

        Ok(SwapExactInIxArgs {
            amount: *amt,
            limit: *limit,
            accs,
            inp_lst_index,
            out_lst_index,
        })
    }

    #[inline]
    pub fn swap_exact_out_ix(
        &mut self,
        TradeIxArgs {
            amt,
            limit,
            mints,
            signer,
            token_accs:
                Pair {
                    inp: inp_token_acc,
                    out: out_token_acc,
                },
        }: &TradeIxArgs,
    ) -> Result<SwapExactOutIxArgsStd, InfErr> {
        let Pair {
            out: out_mint,
            inp: inp_mint,
        } = mints;
        let (
            (inp_lst_index, inp_lst_state, inp_calc, inp_reserves),
            (out_lst_index, out_lst_state, out_calc, out_reserves),
            pfa,
        ) = self.swap_ix_common(mints)?;
        let pricing = self
            .pricing
            .price_exact_out_accs_for(mints)
            .map_err(InfErr::PricingProg)?;

        let accs = SwapExactOutIxAccs {
            ix_prefix: NewSwapExactOutIxPreAccsBuilder::start()
                .with_inp_pool_reserves(inp_reserves)
                .with_out_pool_reserves(out_reserves)
                .with_protocol_fee_accumulator(pfa)
                .with_signer(**signer)
                .with_inp_lst_acc(**inp_token_acc)
                .with_out_lst_acc(**out_token_acc)
                .with_inp_lst_mint(**inp_mint)
                .with_out_lst_mint(**out_mint)
                .with_inp_lst_token_program(TOKEN_PROGRAM)
                .with_out_lst_token_program(TOKEN_PROGRAM)
                .with_lst_state_list(LST_STATE_LIST_ID)
                .with_pool_state(POOL_STATE_ID)
                .build(),
            pricing_prog: self.pool.pricing_program,
            pricing,
            inp_calc_prog: inp_lst_state.sol_value_calculator,
            inp_calc,
            out_calc_prog: out_lst_state.sol_value_calculator,
            out_calc,
        };

        Ok(SwapExactOutIxArgs {
            amount: *amt,
            limit: *limit,
            accs,
            inp_lst_index,
            out_lst_index,
        })
    }
}

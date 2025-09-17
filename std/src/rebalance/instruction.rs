use inf1_core::{
    inf1_ctl_core::{
        instructions::rebalance::{
            end::EndRebalanceIxPreKeysOwned,
            start::{NewStartRebalanceIxPreAccsBuilder, StartRebalanceIxPreKeysOwned},
        },
        keys::{INSTRUCTIONS_SYSVAR_ID, LST_STATE_LIST_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
    },
    instructions::rebalance::{
        end::EndRebalanceIxAccs,
        start::{StartRebalanceIxAccs, StartRebalanceIxArgs},
    },
};
use inf1_pp_ag_std::update::all::Pair;
use inf1_svc_ag_std::{
    inf1_svc_lido_core::solido_legacy_core::SYSTEM_PROGRAM,
    inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::TOKEN_PROGRAM,
    instructions::SvcCalcAccsAg,
};

use crate::{err::InfErr, Inf};

pub type StartRebalanceIxArgsStd =
    StartRebalanceIxArgs<[u8; 32], StartRebalanceIxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg>;

pub type EndRebalanceIxAccsStd =
    EndRebalanceIxAccs<[u8; 32], EndRebalanceIxPreKeysOwned, SvcCalcAccsAg>;

pub struct RebalanceIxArgs<'a> {
    pub out_amt: u64,
    pub min_starting_out_lst: u64,
    pub max_starting_inp_lst: u64,
    pub mints: &'a Pair<&'a [u8; 32]>,
    pub withdraw_to: &'a [u8; 32],
}

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn rebalance_ixs_mut(
        &mut self,
        RebalanceIxArgs {
            out_amt,
            min_starting_out_lst,
            max_starting_inp_lst,
            mints,
            withdraw_to,
        }: &RebalanceIxArgs,
    ) -> Result<(StartRebalanceIxArgsStd, EndRebalanceIxAccsStd), InfErr> {
        let Pair {
            inp: (inp_lst_index, inp_state, inp_calc, inp_reserves),
            out: (out_lst_index, out_state, out_calc, out_reserves),
        } = mints.try_map(|m| self.lst_vars_mut(m))?;
        let start = StartRebalanceIxArgsStd {
            out_lst_index,
            inp_lst_index,
            amount: *out_amt,
            min_starting_out_lst: *min_starting_out_lst,
            max_starting_inp_lst: *max_starting_inp_lst,
            accs: StartRebalanceIxAccs {
                ix_prefix: NewStartRebalanceIxPreAccsBuilder::start()
                    .with_inp_lst_mint(*mints.inp)
                    .with_inp_pool_reserves(inp_reserves)
                    .with_instructions(INSTRUCTIONS_SYSVAR_ID)
                    .with_lst_state_list(LST_STATE_LIST_ID)
                    .with_out_lst_mint(*mints.out)
                    .with_out_lst_token_program(TOKEN_PROGRAM)
                    .with_out_pool_reserves(out_reserves)
                    .with_pool_state(POOL_STATE_ID)
                    .with_rebalance_auth(self.pool.rebalance_authority)
                    .with_rebalance_record(REBALANCE_RECORD_ID)
                    .with_system_program(SYSTEM_PROGRAM)
                    .with_withdraw_to(**withdraw_to)
                    .build(),
                out_calc_prog: out_state.sol_value_calculator,
                out_calc,
                inp_calc_prog: inp_state.sol_value_calculator,
                inp_calc,
            },
        };
        let end = EndRebalanceIxAccsStd::from_start(start.accs);
        Ok((start, end))
    }
}

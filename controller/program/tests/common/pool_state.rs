use std::borrow::Borrow;

use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2, sync_sol_val::SyncSolVal, typedefs::snap::NewSnapBuilder,
};
use inf1_svc_jiminy::traits::SolValCalc;

/// Calc, balance, sol value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cbs<C> {
    pub calc: C,
    pub balance: u64,
    pub old_sol_val: u64,
}

/// Lookaheads PoolStateV2 to after the initial sync for most instructions which
/// usually involves
/// - release yield
/// - zero or more SyncSolVal + UpdateYield
pub fn header_lookahead<'a, I, R, C>(mut ps: PoolStateV2, lsts: I, curr_slot: u64) -> PoolStateV2
where
    I: IntoIterator<Item = R>,
    R: Borrow<Cbs<C>>,
    C: SolValCalc + 'a,
{
    ps.release_yield(curr_slot).unwrap();
    lsts.into_iter().for_each(|c| {
        let Cbs {
            calc,
            balance,
            old_sol_val,
        } = c.borrow();
        ps.apply_ssv_uy(&SyncSolVal {
            lst_sol_val: NewSnapBuilder::start()
                .with_old(*old_sol_val)
                .with_new(*calc.lst_to_sol(*balance).unwrap().start())
                .build(),
        })
        .unwrap();
    });
    ps
}

pub fn assert_lp_solvent_invar(ps: &PoolStateV2) {
    assert!(ps.total_sol_value >= ps.withheld_lamports + ps.protocol_fee_lamports);
}

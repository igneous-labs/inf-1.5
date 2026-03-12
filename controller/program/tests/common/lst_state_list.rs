use inf1_ctl_jiminy::typedefs::lst_state::LstState;
use inf1_svc_jiminy::traits::SolValCalc;

/// For use when sol value in LstState `s` is stale
pub fn lst_state_lookahead(mut s: LstState, balance: u64, calc: impl SolValCalc) -> LstState {
    let new = *calc.lst_to_sol(balance).unwrap().start();
    s.sol_value = new;
    s
}

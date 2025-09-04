use jupiter_amm_interface::{AmmContext, ClockRef};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref AMM_CONTEXT: AmmContext = {
        AmmContext {
            clock_ref: ClockRef::default(),
        }
    };
}

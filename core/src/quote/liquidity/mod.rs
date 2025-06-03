use sanctum_fee_ratio::{
    ratio::{Ceil, Ratio},
    Fee,
};

pub mod add;
pub mod remove;

type Lppf = Fee<Ceil<Ratio<u16, u16>>>;

pub const fn lp_protocol_fee(lp_protocol_fee_bps: u16) -> Option<Lppf> {
    Lppf::new(Ratio {
        n: lp_protocol_fee_bps,
        d: 10_000,
    })
}

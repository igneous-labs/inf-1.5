use sanctum_fee_ratio::{
    ratio::{Ceil, Ratio},
    Fee,
};

pub const BPS_DENOM: u16 = 10_000;

pub type FeeBps = Fee<Ceil<Ratio<u16, u16>>>;

#[inline]
pub const fn fee_bps(bps: u16) -> Option<FeeBps> {
    FeeBps::new(Ratio {
        n: bps,
        d: BPS_DENOM,
    })
}

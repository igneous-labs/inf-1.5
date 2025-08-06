mod deprecated;
mod main;

pub use deprecated::*;
pub use main::*;

use crate::PricingAg;

impl<A, FlatFee> AsRef<A> for PricingAg<FlatFee>
where
    A: ?Sized,
    FlatFee: AsRef<A>,
{
    #[inline]
    fn as_ref(&self) -> &A {
        match self {
            Self::FlatFee(g) => g.as_ref(),
        }
    }
}

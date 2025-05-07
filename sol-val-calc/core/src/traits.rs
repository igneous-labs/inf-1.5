use core::ops::RangeInclusive;

pub trait SolValueCalculator {
    type Error;

    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error>;
}

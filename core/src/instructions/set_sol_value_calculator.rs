#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetSolValueCalculatorIxAccs<T, I, C> {
    pub ix_prefix: I,
    pub calc_prog: T,
    pub calc: C,
}

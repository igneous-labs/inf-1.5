use super::{interface, Abr, AccountHandle, ConstAccs, ConstPdas, ProgramError, SolValCalc};

pub trait GenSvcProgram {
    type Calc: SolValCalc;

    type ProgErr: core::error::Error + Into<ProgramError>;

    /// Should also verify the correctness of accs and return
    /// Err if wrong
    fn try_derive_calc(
        &self,
        abr: &mut Abr,
        accs: &interface::IxAccsGen<AccountHandle>,
        amt: u64,
    ) -> Result<Self::Calc, Self::ProgErr>;

    fn conv_calc_err(&self, e: <Self::Calc as SolValCalc>::Error) -> ProgramError;

    fn const_keys(&self) -> ConstAccs<[u8; 32]>;

    fn const_pdas(&self) -> ConstPdas<([u8; 32], u8)>;
}

use inf1_svc_generic_jiminy::traits::SolValCalc;

use super::{interface, keys::ConstAccs, pda::ConstPdas, Abr, AccountHandle, ProgramError};

pub trait GenSvcProgram {
    type Calc: SolValCalc;

    /// Should also verify the correctness of accs and return
    /// Err if wrong
    fn try_derive_calc(
        &self,
        abr: &mut Abr,
        accs: &interface::IxAccsGen<AccountHandle>,
        amt: u64,
    ) -> Result<Self::Calc, ProgramError>;

    fn conv_calc_err(&self, e: <Self::Calc as SolValCalc>::Error) -> ProgramError;

    fn const_keys_owned(&self) -> ConstAccs<[u8; 32]>;

    fn const_pdas(&self) -> ConstPdas<([u8; 32], u8)>;
}

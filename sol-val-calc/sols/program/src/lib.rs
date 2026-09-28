use inf1_svc_generic_program::{
    instructions::interface::IxAccsGen, keys::ConstAccs, pda::ConstPdas, program::GenSvcProgram,
    traits::SolValCalc, Abr, AccountHandle, ProgramError,
};
use inf1_svc_sols_core::{
    calc::SolsCalc,
    keys::{CONST_KEYS_OWNED, CONST_PDAS},
};

pub struct SolsGenSvcProg;

impl GenSvcProgram for SolsGenSvcProg {
    type Calc = SolsCalc;

    #[inline]
    fn try_derive_calc(
        &self,
        _abr: &mut Abr,
        _accs: &IxAccsGen<AccountHandle>,
        _amt: u64,
    ) -> Result<Self::Calc, ProgramError> {
        //let ps = try_pool_v1(acc)

        todo!()
    }

    #[inline]
    fn conv_calc_err(&self, _e: <Self::Calc as SolValCalc>::Error) -> ProgramError {
        todo!()
    }

    #[inline]
    fn const_keys_owned(&self) -> ConstAccs<[u8; 32]> {
        CONST_KEYS_OWNED
    }

    #[inline]
    fn const_pdas(&self) -> ConstPdas<([u8; 32], u8)> {
        CONST_PDAS
    }
}

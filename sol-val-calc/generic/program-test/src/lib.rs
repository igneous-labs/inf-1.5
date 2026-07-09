#![allow(unexpected_cfgs)]

use inf1_svc_generic_program::{
    instructions::interface::IxAccsGen,
    keys::{ConstAccs, ConstAccsDestr},
    pda::ConstPdas,
    program::GenSvcProgram,
    traits::SolValCalc,
    Abr, AccountHandle, ProgramError,
};
use inf1_svc_wsol_core::calc::WsolCalc;
use jiminy_entrypoint::entrypoint;

/// A test [`GenSvcProgram`] implementor that is deployed
/// to the sanctum-spl-multi sol val calc prog addr but simply
/// echoes `amt..=amt` for both interface methods,
/// since we are only using this to test the common
/// generic instructions and functionality
pub struct MockGenSvc;

pub const CONST_KEY_STRS: ConstAccs<&str> = ConstAccs::const_from_destr(ConstAccsDestr {
    program: "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo",
    pool_prog: "SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn",
    init_manager: "2YVM6H6qZBF8TNMErw11nVk5qA1ZnNBDVw32bCpFc1em",
});

pub const CONST_KEYS_OWNED: ConstAccs<[u8; 32]> = CONST_KEY_STRS.const_keys();

pub const CONST_PDAS: ConstPdas<([u8; 32], u8)> =
    ConstPdas::const_find_from_const_accs(&CONST_KEYS_OWNED);

impl GenSvcProgram for MockGenSvc {
    type Calc = WsolCalc;

    #[inline]
    fn try_derive_calc(
        &self,
        _abr: &mut Abr,
        _accs: &IxAccsGen<AccountHandle>,
        _amt: u64,
    ) -> Result<Self::Calc, ProgramError> {
        Ok(WsolCalc)
    }

    #[inline]
    fn conv_calc_err(&self, _e: <Self::Calc as SolValCalc>::Error) -> ProgramError {
        // e is Infallible
        unreachable!()
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

entrypoint!(process_ix);

#[inline]
fn process_ix(
    abr: &mut Abr,
    accs: &[AccountHandle<'_>],
    data: &[u8],
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    inf1_svc_generic_program::process_ix(abr, accs, data, &MockGenSvc)
}

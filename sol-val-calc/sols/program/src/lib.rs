#![allow(unexpected_cfgs)]

use core::mem::MaybeUninit;

use inf1_svc_generic_program::{
    instructions::interface::IxAccsGen, keys::ConstAccs, pda::ConstPdas, program::GenSvcProgram,
    traits::SolValCalc, verify::verify_pks, Abr, AccountHandle, ProgramError,
};
use inf1_svc_sols_core::{
    calc::SolsCalc,
    keys::{CONST_KEYS_OWNED, CONST_PDAS},
};
use jiminy_entrypoint::entrypoint;
use sanctum_sols_jiminy::{
    instructions::common::{MintPoolPair, MintPoolPairDestr},
    onchain::{accounts::try_pool_v1, pda::prog_create_raw_pool_pda_to},
};

pub struct SolsGenSvcProg;

impl GenSvcProgram for SolsGenSvcProg {
    type Calc = SolsCalc;

    #[inline]
    fn try_derive_calc(
        &self,
        abr: &mut Abr,
        accs: &IxAccsGen<AccountHandle>,
        _amt: u64,
    ) -> Result<Self::Calc, ProgramError> {
        let (pool, _) = try_pool_v1(abr.get(*accs.suf.pool_state()))?;

        let expected_mint = pool.addrs.mint();
        let mut expected_pool = MaybeUninit::uninit();
        let expected_pool =
            prog_create_raw_pool_pda_to(expected_mint, pool.bumps.pool(), &mut expected_pool)?;

        verify_pks(
            abr,
            &MintPoolPair::from_destr(MintPoolPairDestr {
                mint: *accs.pre.lst_mint(),
                pool: *accs.suf.pool_state(),
            })
            .0,
            &MintPoolPair::from_destr(MintPoolPairDestr {
                mint: expected_mint,
                pool: expected_pool,
            })
            .0,
        )?;

        // other accs verified in inf1-svc-generic-program

        Ok(SolsCalc)
    }

    #[inline]
    fn conv_calc_err(&self, infallible: <Self::Calc as SolValCalc>::Error) -> ProgramError {
        match infallible {}
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
    inf1_svc_generic_program::process_ix(abr, accs, data, &SolsGenSvcProg)
}

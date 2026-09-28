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

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use solana_pubkey::Pubkey;

    use super::*;

    #[test]
    fn const_addrs_snapshot() {
        let svc = SolsGenSvcProg;
        expect![[r#"
            ConstAccsDestr {
                program: sssQe6fXL4KRDGeGvoFULZakZjwQ1DKd7vu29QDJBxP,
                pool_prog: so1f7APRw5pJ5iNJrM9g5X9tQgXzk8kMUnXxDNdt99b,
                init_manager: 7eYqVNDg6kkDaWwNkGnMgQX9Vsn4yuc9zyx2t5n6AG5p,
            }
        "#]]
        .assert_debug_eq(&ConstAccs(svc.const_keys_owned().0.map(Pubkey::from)).into_destr());
        expect![[r#"
            ConstPdasDestr {
                state: (
                    8sPPG73ivU1b4Pt4fkdX5HytrGmJEL3um6gT5RdSWSvq,
                    255,
                ),
                pool_progdata: (
                    4MhhkFGvNX24QpAkRPWn2f3CDPjQEmbd1V9QxHiS3Dsy,
                    255,
                ),
            }
        "#]]
        .assert_debug_eq(
            &ConstPdas(
                svc.const_pdas()
                    .0
                    .map(|(addr, bump)| (Pubkey::from(addr), bump)),
            )
            .into_destr(),
        );
    }
}

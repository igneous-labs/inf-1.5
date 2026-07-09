#![allow(unexpected_cfgs)]

use inf1_ctl_jiminy::{
    account_utils::pool_state_v2_checked,
    program_err::Inf1CtlCustomProgErr,
    svc::{InfCalc, InfCalcErr},
    yields::release::ReleaseYieldParams,
};
use inf1_svc_generic_program::{
    errs::GenSvcErr,
    instructions::interface::IxAccsGen,
    keys::{ConstAccs, ConstAccsDestr},
    pda::ConstPdas,
    program::GenSvcProgram,
    program_err::GenSvcProgErr,
    traits::SolValCalc,
    verify::verify_pks,
    Abr, AccountHandle, ProgramError,
};
use jiminy_entrypoint::{account::Account, entrypoint};
use jiminy_sysvar_clock::{program_error::INVALID_ACCOUNT_DATA, sysvar::SimpleSysvar, Clock};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::mint::{Mint, RawMint};

pub struct InfGenSvcProg;

pub const CONST_KEY_STRS: ConstAccs<&str> = ConstAccs::const_from_destr(ConstAccsDestr {
    program: "1nf7dspGYz1CTALJbtNgjvcSYWiFz5N3c2EuUZLSWCL",
    pool_prog: "5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx",
    init_manager: "7eYqVNDg6kkDaWwNkGnMgQX9Vsn4yuc9zyx2t5n6AG5p",
});

pub const CONST_KEYS_OWNED: ConstAccs<[u8; 32]> = CONST_KEY_STRS.const_keys();

pub const CONST_PDAS: ConstPdas<([u8; 32], u8)> =
    ConstPdas::const_find_from_const_accs(&CONST_KEYS_OWNED);

pub const INF_MINT_ID_STR: &str = "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm";
pub const INF_MINT_ID: [u8; 32] = const_crypto::bs58::decode_pubkey(INF_MINT_ID_STR);

pub const POOL_STATE_ID_STR: &str = inf1_ctl_jiminy::pda::CONST_PDA_KEY_STRS.pool_state();
pub const POOL_STATE_ID: [u8; 32] = *inf1_ctl_jiminy::pda::CONST_PDA_KEYS_OWNED.pool_state();

impl GenSvcProgram for InfGenSvcProg {
    type Calc = InfCalc;

    #[inline]
    fn try_derive_calc(
        &self,
        abr: &mut Abr,
        accs: &IxAccsGen<AccountHandle>,
        _amt: u64,
    ) -> Result<Self::Calc, ProgramError> {
        verify_pks(
            abr,
            &[*accs.pre.lst_mint(), *accs.suf.pool_state()],
            &[&INF_MINT_ID, &POOL_STATE_ID],
        )?;
        // other accs verified in inf1-svc-generic-program

        let pool_state = pool_state_v2_checked(abr.get(*accs.suf.pool_state()))?;
        let lst_mint_supply = checked_mint_of(abr.get(*accs.pre.lst_mint()))?.supply();

        let calc = InfCalc::new(pool_state, lst_mint_supply);
        let curr_slot = Clock::get()?.slot;
        calc.lookahead(
            ReleaseYieldParams::new(pool_state, curr_slot).map_err(Inf1CtlCustomProgErr)?,
        )
        .ok_or_else(|| GenSvcProgErr(GenSvcErr::MathError).into())
    }

    #[inline]
    fn conv_calc_err(&self, e: <Self::Calc as SolValCalc>::Error) -> ProgramError {
        match e {
            InfCalcErr::Math => GenSvcProgErr(GenSvcErr::MathError).into(),
        }
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

/// `_checked` because it also verifies that the acc is properly initialized.
///
/// Compatible with token-22
#[inline]
pub fn checked_mint_of(acc: &Account) -> Result<Mint<'_>, ProgramError> {
    Ok(acc
        .data()
        .first_chunk() // ignore token-22 extension data
        .map(RawMint::of_acc_data_arr)
        .and_then(Mint::try_from_raw)
        .ok_or(INVALID_ACCOUNT_DATA)?)
}

entrypoint!(process_ix);

#[inline]
fn process_ix(
    abr: &mut Abr,
    accs: &[AccountHandle<'_>],
    data: &[u8],
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    inf1_svc_generic_program::process_ix(abr, accs, data, &InfGenSvcProg)
}

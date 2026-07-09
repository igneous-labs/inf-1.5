use inf1_ctl_jiminy::{
    err::Inf1CtlErr,
    instructions::init::{InitIxPreAccs, InitIxPreAccsDestr, INIT_IX_PRE_IS_SIGNER},
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::{
    token::checked_mint_of,
    utils::accs_split_first_chunk,
    verify::{verify_pks, verify_pks_raw, verify_signers},
    Cpi,
};

const ALLOWED_DECIMALS: u8 = 9;

#[inline]
pub fn init_pre_accs_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<InitIxPreAccs<AccountHandle<'acc>>, ProgramError> {
    let (accs, _) = accs_split_first_chunk(accs)?;
    let accs = InitIxPreAccs(*accs);

    verify_pks(
        abr,
        &accs.0,
        &InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            init_admin: CONST_KEYS_OWNED.init_admin(),
            pool_state: CONST_PDA_KEYS_OWNED.pool_state(),

            // Free: mint properties verified below
            lp_token_mint: abr.get(*accs.lp_token_mint()).key(),

            // Free: payer can be anyone, no lof since we never invoke_signed with payer
            payer: abr.get(*accs.payer()).key(),
        })
        .0,
    )?;

    verify_signers(abr, &accs.0, &INIT_IX_PRE_IS_SIGNER.0)?;

    verify_pks_raw(
        &[accs.pool_state(), accs.lp_token_mint()].map(|h| abr.get(*h).owner()),
        &[CONST_KEYS_OWNED.sys_prog(), CONST_KEYS_OWNED.tokenkeg()],
    )?;

    let mint = checked_mint_of(abr.get(*accs.lp_token_mint()))?;
    if mint.supply() != 0 || mint.decimals() != ALLOWED_DECIMALS {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::IncorrectLpMintInitialization).into());
    }

    Ok(accs)
}

#[inline]
pub fn process_init(
    _abr: &mut Abr,
    _cpi: &mut Cpi,
    _accs: &InitIxPreAccs<AccountHandle>,
) -> Result<(), ProgramError> {
    todo!()
}

use inf1_pp_reserve_v2_core::{
    instructions::init::{InitIxPreAccs, InitIxPreAccsDestr},
    pda::CONST_PDA_KEYS_OWNED,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::utils::{asfc, verify_pks};

pub type InitIxPreAccHandles<'a> = InitIxPreAccs<AccountHandle<'a>>;

pub fn init_accs_checked<'acc>(
    abr: &Abr,
    accounts: &[AccountHandle<'acc>],
) -> Result<InitIxPreAccs<AccountHandle<'acc>>, ProgramError> {
    let (pre_accs, _) = asfc(accounts)?;
    let accs = InitIxPreAccHandles::new(*pre_accs);

    verify_pks(
        abr,
        &accs.0,
        &InitIxPreAccs::from_destr(InitIxPreAccsDestr {
            pricing_state: CONST_PDA_KEYS_OWNED.pricing_state(),
            payer: abr.get(*accs.payer()).key(),
        })
        .0,
    )?;

    // No need to verify signers, payer will just fail to pay for rent
    // and ix will fail if not signed

    Ok(accs)
}

pub fn process_init<'acc>(
    abr: &mut Abr,
    accs: InitIxPreAccHandles<'acc>,
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    let _ = (abr, accs);
    todo!()
}

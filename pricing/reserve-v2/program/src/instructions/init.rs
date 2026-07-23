use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_account_size,
    init::INITIAL_ENTRIES,
    instructions::init::{InitIxPreAccs, InitIxPreAccsDestr},
    keys::CONST_KEYS_OWNED,
    pda::CONST_PDA_KEYS_OWNED,
};
use inf1_pp_reserve_v2_jiminy::account_utils::pricing_state_checked_mut;
use inf1_pp_reserve_v2_jiminy::pda_onchain::PRICING_STATE_SIGNER;
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};
use sanctum_system_jiminy::{
    instructions::assign::assign_invoke_signed,
    sanctum_system_core::instructions::{
        assign::NewAssignIxAccsBuilder, transfer::NewTransferIxAccsBuilder,
    },
};

use crate::{
    utils::{asfc, pay_for_rent_exempt_shortfall, verify_owners, verify_pks},
    Cpi,
};

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

    verify_owners(
        abr,
        &[*accs.pricing_state()],
        &[CONST_KEYS_OWNED.sys_prog()],
    )?;

    Ok(accs)
}

pub fn process_init<'acc>(
    abr: &mut Abr,
    accs: InitIxPreAccHandles<'acc>,
) -> Result<(), ProgramError> {
    let mut cpi = Cpi::new();

    assign_invoke_signed(
        abr,
        &mut cpi,
        NewAssignIxAccsBuilder::start()
            .with_assign(*accs.pricing_state())
            .build(),
        CONST_KEYS_OWNED.program(),
        &[PRICING_STATE_SIGNER],
    )?;

    let pricing_state = abr.get_mut(*accs.pricing_state());
    pricing_state.realloc(pricing_state_account_size(INITIAL_ENTRIES.len()))?;

    let (admin, entries) = pricing_state_checked_mut(pricing_state)?;
    *admin = *CONST_KEYS_OWNED.init_admin();
    entries.0.copy_from_slice(&INITIAL_ENTRIES);

    pay_for_rent_exempt_shortfall(
        abr,
        &mut cpi,
        NewTransferIxAccsBuilder::start()
            .with_from(*accs.payer())
            .with_to(*accs.pricing_state())
            .build(),
    )?;

    Ok(())
}

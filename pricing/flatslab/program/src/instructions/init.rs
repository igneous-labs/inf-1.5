use inf1_pp_flatslab_core::{
    accounts::{Slab, SlabMut},
    errs::FlatSlabProgramErr,
    instructions::init::{InitIxAccs, InitIxKeys, NewInitIxAccsBuilder},
    keys::{INITIAL_ADMIN_ID, SLAB_BUMP, SLAB_ID},
    pda::SLAB_SEED,
    typedefs::SlabEntryPacked,
};
use jiminy_cpi::{
    pda::{PdaSeed, PdaSigner},
    program_error::{INVALID_ACCOUNT_DATA, INVALID_ARGUMENT, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_entrypoint::{account::AccountHandle, program_error::ProgramError};
use jiminy_system_prog_interface::{
    assign_ix, transfer_ix, AssignIxData, NewAssignIxAccsBuilder, NewTransferIxAccsBuilder,
    TransferIxData,
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};

use crate::{
    utils::{verify_pks, Cpi, SYS_PROG_ID},
    Accounts, CustomProgErr,
};

pub type InitIxAccHandles<'a> = InitIxAccs<AccountHandle<'a>>;

fn expected_init_ix_keys(payer: &[u8; 32]) -> InitIxKeys {
    NewInitIxAccsBuilder::start()
        .with_payer(payer)
        .with_slab(&SLAB_ID)
        .with_system_program(&SYS_PROG_ID)
        .build()
}

pub fn init_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
) -> Result<InitIxAccHandles<'acc>, ProgramError> {
    let Some(init_accs) = accounts.as_slice().first_chunk() else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };
    let accs = InitIxAccHandles::new(*init_accs);
    let payer_key = accounts.get(*accs.payer()).key();
    verify_pks(accounts, &accs.0, &expected_init_ix_keys(payer_key).0).map_err(
        |(_actual, expected)| match *expected {
            SLAB_ID => ProgramError::from(CustomProgErr(FlatSlabProgramErr::WrongSlabAcc)),
            _ => INVALID_ARGUMENT.into(),
        },
    )?;

    // no need to check signers here, rely on system program
    // transfer's CPI's check if required

    Ok(accs)
}

// first entry is that of LP mint
const INIT_ACC_LEN: usize = Slab::account_size(1);

pub fn process_init<'acc>(
    accounts: &mut Accounts<'acc>,
    accs: InitIxAccHandles<'acc>,
    prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    let mut cpi = Cpi::new();

    let slab = accounts.get(*accs.slab());
    if *slab.owner() != SYS_PROG_ID {
        return Err(INVALID_ACCOUNT_DATA.into());
    }

    let lamports_shortfall = Rent::get()?
        .min_balance(INIT_ACC_LEN)
        .saturating_sub(slab.lamports());

    if lamports_shortfall > 0 {
        cpi.invoke_signed(
            accounts,
            transfer_ix(
                *accs.system_program(),
                NewTransferIxAccsBuilder::start()
                    .with_from(*accs.payer())
                    .with_to(*accs.slab())
                    .build(),
                &TransferIxData::new(lamports_shortfall),
            ),
            &[],
        )?;
    }

    cpi.invoke_signed(
        accounts,
        assign_ix(
            *accs.system_program(),
            NewAssignIxAccsBuilder::start()
                .with_assign(*accs.slab())
                .build(),
            &AssignIxData::new(prog_id),
        ),
        &[PdaSigner::new(&[
            PdaSeed::new(&SLAB_SEED),
            PdaSeed::new(&[SLAB_BUMP]),
        ])],
    )?;

    let slab = accounts.get_mut(*accs.slab());
    slab.realloc(INIT_ACC_LEN, false)?;

    let mut slabmut = SlabMut::of_acc_data(slab.data_mut()).ok_or(INVALID_ACCOUNT_DATA)?;
    let (admin, entries) = slabmut.as_mut();

    *admin = INITIAL_ADMIN_ID;
    let entry = entries.0.first_mut().ok_or(INVALID_ACCOUNT_DATA)?;
    *entry = SlabEntryPacked::INITIAL_LP;

    Ok(())
}

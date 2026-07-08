// Re-exports
pub use inf1_svc_generic_jiminy::{instructions::interface, keys, pda, traits::SolValCalc};
pub use jiminy_account::{Abr, AccountHandle};
pub use jiminy_cpi::{program_error::ProgramError, Cpi};

use inf1_svc_generic_jiminy::{
    account_utils::{bpf_loader_v3_programdata_checked, state_checked, state_checked_mut},
    accounts::state::State,
    errs::GenSvcErr,
    instructions::{
        init::{InitIxPreAccs, InitIxPreAccsDestr, INIT_IX_DISCM},
        manager::{
            SetManagerIxAccs, SetManagerIxAccsDestr, ULUSIxAccs, ULUSIxAccsDestr,
            SET_MANAGER_IX_DISCM, SET_MANAGER_IX_IS_SIGNER, ULUS_IX_DISCM, ULUS_IX_IS_SIGNER,
        },
    },
    keys::GLOBAL_CONST_KEYS_OWNED,
    pda_onchain::state_signer,
    program_err::GenSvcProgErr,
};
use jiminy_cpi::{pda::PdaSigner, program_error::INVALID_INSTRUCTION_DATA};
use jiminy_return_data::set_return_data;
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_system_jiminy::{
    instructions::assign::assign_ix_account_handle_perms,
    sanctum_system_core::instructions::{
        assign::{AssignIxData, NewAssignIxAccsBuilder},
        transfer::{NewTransferIxAccsBuilder, TransferIxData},
    },
};

use crate::{
    program::GenSvcProgram,
    utils::{accs_split_first_chunk, ix_data_as_arr},
    verify::{verify_pks, verify_signers},
};

pub mod program;
pub mod utils;
pub mod verify;

/// To create a new generic sol value calculator program,
/// impl [`GenSvcProgram`], then use this fn as the entrypoint
#[inline]
pub fn process_ix(
    abr: &mut Abr,
    accs: &[AccountHandle<'_>],
    data: &[u8],
    prog: impl GenSvcProgram,
) -> Result<(), ProgramError> {
    let const_keys = prog.const_keys_owned();
    let const_pdas = prog.const_pdas();

    match data.split_first().ok_or(INVALID_INSTRUCTION_DATA)? {
        // interface ixs
        (&interface_discm, data)
            if interface_discm == interface::lst_to_sol::LST_TO_SOL_IX_DISCM
                || interface_discm == interface::sol_to_lst::SOL_TO_LST_IX_DISCM =>
        {
            let amt = u64::from_le_bytes(*ix_data_as_arr(data)?);
            let (pre, accs) = accs_split_first_chunk(accs)?;
            let (suf, _) = accs_split_first_chunk(accs)?;
            let accs = interface::IxAccs {
                pre: interface::IxPreAccs(*pre),
                suf: interface::IxSufAccs(*suf),
            };

            // no verify_pks for prefix: responsibility of try_derive_calc
            // to verify identity of lst_mint

            verify_pks(
                abr,
                &accs.suf.0,
                &interface::IxSufAccs::from_destr(interface::IxSufAccsDestr {
                    state: &const_pdas.state().0,
                    pool_prog: const_keys.pool_prog(),
                    pool_progdata: &const_pdas.pool_progdata().0,
                    // Free: responsibility of try_derive_calc to verify pool_state,
                    // usually in relation to pre.lst_mint()
                    pool_state: abr.get(*accs.suf.pool_state()).key(),
                })
                .0,
            )?;

            // no signer verification, permissionless ix

            let state = state_checked(abr.get(*accs.suf.state()))?;
            let lus = bpf_loader_v3_programdata_checked(abr.get(*accs.suf.pool_progdata()))?.0;
            if state.last_upgrade_slot != lus {
                return Err(GenSvcProgErr(GenSvcErr::UnexpectedProgramUpgrade).into());
            }

            let calc = prog.try_derive_calc(abr, &accs, amt)?;

            let res = if interface_discm == interface::lst_to_sol::LST_TO_SOL_IX_DISCM {
                calc.lst_to_sol(amt)
            } else {
                calc.sol_to_lst(amt)
            }
            .map_err(|e| prog.conv_calc_err(e))?;

            set_return_data(&interface::to_retdata(&res));

            Ok(())
        }

        // manager ixs
        (&ULUS_IX_DISCM, _) => {
            let (accs, _) = accs_split_first_chunk(accs)?;
            let accs = ULUSIxAccs(*accs);

            let state = state_checked(abr.get(*accs.state()))?;

            verify_pks(
                abr,
                &accs.0,
                &ULUSIxAccs::from_destr(ULUSIxAccsDestr {
                    manager: &state.manager,
                    state: &const_pdas.state().0,
                    pool_prog: const_keys.pool_prog(),
                    pool_progdata: &const_pdas.pool_progdata().0,
                })
                .0,
            )?;

            verify_signers(abr, &accs.0, &ULUS_IX_IS_SIGNER.0)?;

            let new_lus = bpf_loader_v3_programdata_checked(abr.get(*accs.pool_progdata()))?.0;
            state_checked_mut(abr.get_mut(*accs.state()))?.last_upgrade_slot = new_lus;

            Ok(())
        }
        (&SET_MANAGER_IX_DISCM, _) => {
            let (accs, _) = accs_split_first_chunk(accs)?;
            let accs = SetManagerIxAccs(*accs);

            let state = state_checked(abr.get(*accs.state()))?;

            let new_manager_pk = *abr.get(*accs.new()).key();
            verify_pks(
                abr,
                &accs.0,
                &SetManagerIxAccs::from_destr(SetManagerIxAccsDestr {
                    curr: &state.manager,
                    state: &const_pdas.state().0,
                    // Free: current manager is allowed to set new manager to any key
                    new: &new_manager_pk,
                })
                .0,
            )?;

            verify_signers(abr, &accs.0, &SET_MANAGER_IX_IS_SIGNER.0)?;

            state_checked_mut(abr.get_mut(*accs.state()))?.manager = new_manager_pk;

            Ok(())
        }
        (&INIT_IX_DISCM, _) => {
            let (accs, _) = accs_split_first_chunk(accs)?;
            let accs = InitIxPreAccs(*accs);

            verify_pks(
                abr,
                &accs.0,
                &InitIxPreAccs::from_destr(InitIxPreAccsDestr {
                    state: &const_pdas.state().0,
                    // Free: any payer acceptable, no loss of funds
                    // since we never invoke transfer signed
                    payer: abr.get(*accs.payer()).key(),
                })
                .0,
            )?;

            // no signer verification required, rely on sys transfer failing
            // if payer not a signer

            if abr.get(*accs.state()).owner() != GLOBAL_CONST_KEYS_OWNED.sys_prog() {
                return Err(GenSvcProgErr(GenSvcErr::StateAlreadyInitialized).into());
            }

            // only need max 2 accs for transfer
            let mut cpi: Cpi<2> = Cpi::new();
            let ss = state_signer(&const_pdas.state().1);
            let ss = PdaSigner::new(&ss);
            let rent = Rent::get()?;
            let state_lamports = abr.get(*accs.state()).lamports();

            cpi.invoke_signed(
                abr,
                GLOBAL_CONST_KEYS_OWNED.sys_prog(),
                AssignIxData::new(const_keys.program()).as_buf(),
                assign_ix_account_handle_perms(
                    NewAssignIxAccsBuilder::start()
                        .with_assign(*accs.state())
                        .build(),
                ),
                &[ss],
            )?;
            let lamports_shortfall = rent
                .min_balance(core::mem::size_of::<State>())
                .saturating_sub(state_lamports);
            if lamports_shortfall > 0 {
                cpi.invoke_fwd(
                    abr,
                    GLOBAL_CONST_KEYS_OWNED.sys_prog(),
                    TransferIxData::new(lamports_shortfall).as_buf(),
                    NewTransferIxAccsBuilder::start()
                        .with_from(*accs.payer())
                        .with_to(*accs.state())
                        .build()
                        .0,
                )?;
            }
            abr.get_mut(*accs.state())
                .realloc(core::mem::size_of::<State>(), false)?;

            *state_checked_mut(abr.get_mut(*accs.state()))? = State {
                manager: *const_keys.init_manager(),
                last_upgrade_slot: 0,
            };

            Ok(())
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

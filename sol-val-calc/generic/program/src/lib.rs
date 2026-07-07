// Re-exports
pub use inf1_svc_generic_jiminy::{
    instructions::interface, keys::ConstAccs, pda::ConstPdas, traits::SolValCalc,
};
pub use jiminy_account::{Abr, AccountHandle};
pub use jiminy_cpi::{program_error::ProgramError, Cpi};

use inf1_svc_generic_jiminy::{
    account_utils::{bpf_loader_v3_programdata_checked, state_checked, state_checked_mut},
    instructions::{
        init::INIT_IX_DISCM,
        manager::{
            SetManagerIxAccs, SetManagerIxAccsDestr, ULUSIxAccs, ULUSIxAccsDestr,
            SET_MANAGER_IX_DISCM, SET_MANAGER_IX_IS_SIGNER, ULUS_IX_DISCM, ULUS_IX_IS_SIGNER,
        },
    },
};
use jiminy_cpi::program_error::INVALID_INSTRUCTION_DATA;
use jiminy_return_data::set_return_data;

use crate::{
    program::GenSvcProgram,
    utils::{accs_split_first_chunk, ix_data_as_arr},
    verify::{verify_pks, verify_signers},
};

mod instructions;

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
    let const_keys = prog.const_keys();
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

            let calc = prog.try_derive_calc(abr, &accs, amt).map_err(Into::into)?;

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
            todo!()
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

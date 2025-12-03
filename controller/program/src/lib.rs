#![allow(unexpected_cfgs)]

use std::{alloc::Layout, mem::MaybeUninit};

use inf1_ctl_jiminy::instructions::{
    admin::{
        add_lst::ADD_LST_IX_DISCM,
        lst_input::{disable::DISABLE_LST_INPUT_IX_DISCM, enable::ENABLE_LST_INPUT_IX_DISCM},
        remove_lst::{RemoveLstIxData, REMOVE_LST_IX_DISCM},
        set_admin::SET_ADMIN_IX_DISCM,
        set_pricing_prog::SET_PRICING_PROG_IX_DISCM,
        set_sol_value_calculator::{SetSolValueCalculatorIxData, SET_SOL_VALUE_CALC_IX_DISCM},
    },
    disable_pool::{
        add_disable_pool_auth::ADD_DISABLE_POOL_AUTH_IX_DISCM, disable::DISABLE_POOL_IX_DISCM,
        enable::ENABLE_POOL_IX_DISCM, remove_disable_pool_auth::REMOVE_DISABLE_POOL_AUTH_IX_DISCM,
    },
    liquidity::{
        add::ADD_LIQUIDITY_IX_DISCM, parse_liq_ix_args, remove::REMOVE_LIQUIDITY_IX_DISCM,
    },
    protocol_fee::{
        set_protocol_fee::SET_PROTOCOL_FEE_IX_DISCM,
        set_protocol_fee_beneficiary::SET_PROTOCOL_FEE_BENEFICIARY_IX_DISCM,
        withdraw_protocol_fees::WITHDRAW_PROTOCOL_FEES_IX_DISCM,
    },
    rebalance::{
        end::END_REBALANCE_IX_DISCM,
        set_rebal_auth::SET_REBAL_AUTH_IX_DISCM,
        start::{StartRebalanceIxData, START_REBALANCE_IX_DISCM},
    },
    swap::{
        parse_swap_ix_args,
        v1::{exact_in::SWAP_EXACT_IN_IX_DISCM, exact_out::SWAP_EXACT_OUT_IX_DISCM},
        v2::{exact_in::SWAP_EXACT_IN_V2_IX_DISCM, exact_out::SWAP_EXACT_OUT_V2_IX_DISCM},
    },
    sync_sol_value::{SyncSolValueIxData, SYNC_SOL_VALUE_IX_DISCM},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::INVALID_INSTRUCTION_DATA,
};
use jiminy_entrypoint::{
    allocator::Allogator, default_panic_handler, program_entrypoint, program_error::ProgramError,
};
use jiminy_log::sol_log;
use jiminy_sysvar_clock::Clock;
use jiminy_sysvar_rent::sysvar::SimpleSysvar;

use crate::{
    instructions::{
        admin::{
            add_lst::process_add_lst,
            lst_input::{
                common::set_lst_input_checked, disable::process_disable_lst_input,
                enable::process_enable_lst_input,
            },
            remove_lst::process_remove_lst,
            set_admin::{process_set_admin, set_admin_accs_checked},
            set_pricing_prog::{process_set_pricing_prog, set_pricing_prog_accs_checked},
            set_sol_value_calculator::{
                process_set_sol_value_calculator, set_sol_value_calculator_accs_checked,
            },
        },
        disable_pool::{
            add_disable_pool_auth::{
                add_disable_pool_auth_accs_checked, process_add_disable_pool_auth,
            },
            disable::{disable_pool_accs_checked, process_disable_pool},
            enable::{enable_pool_accs_checked, process_enable_pool},
            remove_disable_pool_auth::{
                process_remove_disable_pool_auth, remove_disable_pool_auth_checked,
            },
        },
        protocol_fee::{
            set_protocol_fee::{process_set_protocol_fee, set_protocol_fee_checked},
            set_protocol_fee_beneficiary::{
                process_set_protocol_fee_beneficiary, set_protocol_fee_beneficiary_accs_checked,
            },
            withdraw_protocol_fee::{
                process_withdraw_protocol_fees, withdraw_protocol_fees_checked,
            },
        },
        rebalance::{
            end::process_end_rebalance,
            set_rebal_auth::{process_set_rebal_auth, set_rebal_auth_accs_checked},
            start::process_start_rebalance,
        },
        swap::{
            v1::{
                add_liq_split_v1_accs_into_v2, conv_add_liq_args, conv_rem_liq_args,
                rem_liq_split_v1_accs_into_v2, swap_split_v1_accs_into_v2,
            },
            v2::{
                process_swap_exact_in_v2, process_swap_exact_out_v2, swap_v2_split_accs,
                verify_swap_v2,
            },
        },
        sync_sol_value::process_sync_sol_value,
    },
    utils::ix_data_as_arr,
};

mod acc_migrations;
mod err;
mod instructions;
mod svc;
mod token;
mod utils;
mod verify;

const MAX_ACCS: usize = 64;

/// Ensure no pricing program or sol value calculator programs require
/// more than this number of accounts for CPI
const MAX_CPI_ACCS: usize = 48;

type Cpi = jiminy_cpi::Cpi<MAX_CPI_ACCS>;

// Compile-time allocation of Cpi buffer

const CONST_ALLOCS: (Allogator, *mut Cpi) = const {
    let (a, cpi_ptr) = Allogator::new().const_alloc(Layout::new::<Cpi>());
    (a, cpi_ptr.cast::<Cpi>())
};

#[cfg(target_os = "solana")]
#[global_allocator]
static ALLOCATOR: Allogator = CONST_ALLOCS.0;

const CPI_PTR: *mut Cpi = CONST_ALLOCS.1;

default_panic_handler!();
program_entrypoint!(process_ix, MAX_ACCS);

#[inline]
fn process_ix(
    abr: &mut Abr,
    accounts: &[AccountHandle<'_>],
    data: &[u8],
    _prog_id: &[u8; 32],
) -> Result<(), ProgramError> {
    // Safety:
    // - even tho ptr is pointing to uninitialized memory,
    //   Cpi type is just wrapper around MaybeUninit::uninit()
    //   (might still be UB, idk).
    //   `as_uninit_mut` only available in nightly.
    let cpi: &'static mut Cpi = unsafe { CPI_PTR.as_mut().unwrap_unchecked() };
    let mut clock = MaybeUninit::uninit();

    match data.split_first().ok_or(INVALID_INSTRUCTION_DATA)? {
        (&SYNC_SOL_VALUE_IX_DISCM, data) => {
            sol_log("SyncSolValue");
            let lst_idx = SyncSolValueIxData::parse_no_discm(ix_data_as_arr(data)?) as usize;
            let clock = Clock::write_to(&mut clock)?;
            process_sync_sol_value(abr, cpi, accounts, lst_idx, clock)
        }
        // core user-facing ixs
        (&SWAP_EXACT_IN_IX_DISCM, data) => {
            sol_log("SwapExactIn");
            let args = parse_swap_ix_args(ix_data_as_arr(data)?);
            let accs = swap_split_v1_accs_into_v2(abr, accounts, &args)?;
            let clock = Clock::write_to(&mut clock)?;
            verify_swap_v2(abr, &accs, &args, clock)?;
            process_swap_exact_in_v2(abr, cpi, &accs, &args, clock)
        }
        (&SWAP_EXACT_OUT_IX_DISCM, data) => {
            sol_log("SwapExactOut");
            let args = parse_swap_ix_args(ix_data_as_arr(data)?);
            let accs = swap_split_v1_accs_into_v2(abr, accounts, &args)?;
            let clock = Clock::write_to(&mut clock)?;
            verify_swap_v2(abr, &accs, &args, clock)?;
            process_swap_exact_out_v2(abr, cpi, &accs, &args, clock)
        }
        (&ADD_LIQUIDITY_IX_DISCM, data) => {
            sol_log("AddLiquidity");
            let args = parse_liq_ix_args(ix_data_as_arr(data)?);
            let accs = add_liq_split_v1_accs_into_v2(abr, accounts, &args)?;
            let args = conv_add_liq_args(args);
            let clock = Clock::write_to(&mut clock)?;
            verify_swap_v2(abr, &accs, &args, clock)?;
            process_swap_exact_in_v2(abr, cpi, &accs, &args, clock)
        }
        (&REMOVE_LIQUIDITY_IX_DISCM, data) => {
            sol_log("RemoveLiquidity");
            let args = parse_liq_ix_args(ix_data_as_arr(data)?);
            let accs = rem_liq_split_v1_accs_into_v2(abr, accounts, &args)?;
            let args = conv_rem_liq_args(args);
            let clock = Clock::write_to(&mut clock)?;
            verify_swap_v2(abr, &accs, &args, clock)?;
            process_swap_exact_in_v2(abr, cpi, &accs, &args, clock)
        }
        // admin ixs
        (&DISABLE_LST_INPUT_IX_DISCM, data) => {
            sol_log("DisableLstInput");
            let (accs, idx) = set_lst_input_checked(abr, accounts, data)?;
            process_disable_lst_input(abr, &accs, idx)
        }
        (&ENABLE_LST_INPUT_IX_DISCM, data) => {
            sol_log("EnableLstInput");
            let (accs, idx) = set_lst_input_checked(abr, accounts, data)?;
            process_enable_lst_input(abr, &accs, idx)
        }
        (&ADD_LST_IX_DISCM, _data) => {
            sol_log("AddLst");
            process_add_lst(abr, accounts, cpi)
        }
        (&REMOVE_LST_IX_DISCM, data) => {
            sol_log("RemoveLst");
            let lst_idx = RemoveLstIxData::parse_no_discm(ix_data_as_arr(data)?) as usize;
            process_remove_lst(abr, cpi, accounts, lst_idx)
        }
        (&SET_SOL_VALUE_CALC_IX_DISCM, data) => {
            sol_log("SetSolValueCalculator");
            let lst_idx =
                SetSolValueCalculatorIxData::parse_no_discm(ix_data_as_arr(data)?) as usize;
            let accs = set_sol_value_calculator_accs_checked(abr, accounts, lst_idx)?;
            process_set_sol_value_calculator(abr, cpi, &accs, lst_idx)
        }
        (&SET_ADMIN_IX_DISCM, _) => {
            sol_log("SetAdmin");
            let accs = set_admin_accs_checked(abr, accounts)?;
            process_set_admin(abr, accs)
        }
        (&SET_PRICING_PROG_IX_DISCM, _) => {
            sol_log("SetPricingProg");
            let accs = set_pricing_prog_accs_checked(abr, accounts)?;
            process_set_pricing_prog(abr, accs)
        }
        // protocol fees
        (&SET_PROTOCOL_FEE_IX_DISCM, data) => {
            sol_log("SetProtocolFee");
            let (accs, protocol_fee_nanos) = set_protocol_fee_checked(abr, accounts, data)?;
            process_set_protocol_fee(abr, &accs, protocol_fee_nanos)
        }
        (&SET_PROTOCOL_FEE_BENEFICIARY_IX_DISCM, _) => {
            sol_log("SetProtocolFeeBeneficiary");
            let accs = set_protocol_fee_beneficiary_accs_checked(abr, accounts)?;
            process_set_protocol_fee_beneficiary(abr, accs)
        }
        (&WITHDRAW_PROTOCOL_FEES_IX_DISCM, data) => {
            sol_log("WithdrawProtocolFees");
            let (accs, amt) = withdraw_protocol_fees_checked(abr, accounts, data)?;
            process_withdraw_protocol_fees(abr, cpi, &accs, amt)
        }
        // disable pool system
        (&ADD_DISABLE_POOL_AUTH_IX_DISCM, _) => {
            sol_log("AddDisablePoolAuth");
            let accs = add_disable_pool_auth_accs_checked(abr, accounts)?;
            process_add_disable_pool_auth(abr, cpi, &accs)
        }
        (&REMOVE_DISABLE_POOL_AUTH_IX_DISCM, data) => {
            sol_log("RemoveDisablePoolAuth");
            let (accs, idx) = remove_disable_pool_auth_checked(abr, accounts, data)?;
            process_remove_disable_pool_auth(abr, &accs, idx)
        }
        (&DISABLE_POOL_IX_DISCM, _) => {
            sol_log("DisablePool");
            let accs = disable_pool_accs_checked(abr, accounts)?;
            process_disable_pool(abr, &accs)
        }
        (&ENABLE_POOL_IX_DISCM, _) => {
            sol_log("EnablePool");
            let accs = enable_pool_accs_checked(abr, accounts)?;
            process_enable_pool(abr, &accs)
        }
        // rebalance
        (&START_REBALANCE_IX_DISCM, data) => {
            sol_log("StartRebalance");
            let args = StartRebalanceIxData::parse_no_discm(ix_data_as_arr(data)?);
            process_start_rebalance(abr, accounts, args, cpi)
        }
        (&END_REBALANCE_IX_DISCM, _data) => {
            sol_log("EndRebalance");
            process_end_rebalance(abr, accounts, cpi)
        }
        (&SET_REBAL_AUTH_IX_DISCM, _) => {
            sol_log("SetRebalAuth");
            let accs = set_rebal_auth_accs_checked(abr, accounts)?;
            process_set_rebal_auth(abr, accs)
        }
        // v2 swap
        (&SWAP_EXACT_IN_V2_IX_DISCM, data) => {
            sol_log("SwapExactInV2");
            let args = parse_swap_ix_args(ix_data_as_arr(data)?);
            let accs = swap_v2_split_accs(abr, accounts, &args)?;
            let clock = Clock::write_to(&mut clock)?;
            verify_swap_v2(abr, &accs, &args, clock)?;
            process_swap_exact_in_v2(abr, cpi, &accs, &args, clock)
        }
        (&SWAP_EXACT_OUT_V2_IX_DISCM, data) => {
            sol_log("SwapExactOutV2");
            let args = parse_swap_ix_args(ix_data_as_arr(data)?);
            let accs = swap_v2_split_accs(abr, accounts, &args)?;
            let clock = Clock::write_to(&mut clock)?;
            verify_swap_v2(abr, &accs, &args, clock)?;
            process_swap_exact_out_v2(abr, cpi, &accs, &args, clock)
        }
        _ => Err(INVALID_INSTRUCTION_DATA.into()),
    }
}

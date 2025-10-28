use crate::{
    verify::{
        log_and_return_acc_privilege_err, verify_not_rebalancing_and_not_disabled, verify_pks,
        verify_signers, verify_sol_value_calculator_is_program, verify_tokenkeg_or_22_mint,
    },
    Cpi,
};
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolState,
    },
    err::Inf1CtlErr,
    instructions::admin::add_lst::{AddLstIxAccs, NewAddLstIxAccsBuilder, ADD_LST_IX_IS_SIGNER},
    keys::{ATOKEN_ID, LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID, SYS_PROG_ID},
    pda::{const_find_pool_reserves, const_find_protocol_fee_accumulator},
    program_err::Inf1CtlCustomProgErr,
    typedefs::lst_state::LstState,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_ata_jiminy::sanctum_ata_core::instructions::create::{
    CreateIxData, NewCreateIxAccsBuilder,
};
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::{
    NewTransferIxAccsBuilder, TransferIxData,
};

#[inline]
pub fn process_add_lst(
    abr: &mut Abr,
    accounts: &[AccountHandle],
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let accs = accounts.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = AddLstIxAccs(*accs);

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(abr.get(*accs.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let lst_mint_acc = abr.get(*accs.lst_mint());
    let mint = *lst_mint_acc.key();
    let token_prog = lst_mint_acc.owner();

    let (expected_pool_reserves, pool_reserves_bump) =
        const_find_pool_reserves(token_prog, lst_mint_acc.key());
    let (expected_protocol_fee_accumulator, protocol_fee_accumulator_bump) =
        const_find_protocol_fee_accumulator(token_prog, lst_mint_acc.key());

    let expected_pks = NewAddLstIxAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_payer(abr.get(*accs.payer()).key())
        .with_lst_mint(lst_mint_acc.key())
        .with_pool_reserves(&expected_pool_reserves)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_protocol_fee_accumulator_auth(&PROTOCOL_FEE_ID)
        .with_sol_value_calculator(abr.get(*accs.sol_value_calculator()).key())
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_associated_token_program(&ATOKEN_ID)
        .with_system_program(&SYS_PROG_ID)
        .with_lst_token_program(token_prog)
        .build();

    verify_pks(abr, &accs.0, &expected_pks.0)?;
    verify_signers(abr, &accs.0, &ADD_LST_IX_IS_SIGNER.0)
        .map_err(|expected_signer| log_and_return_acc_privilege_err(abr, *expected_signer))?;

    verify_tokenkeg_or_22_mint(lst_mint_acc)?;
    verify_sol_value_calculator_is_program(abr.get(*accs.sol_value_calculator()))?;

    // Verify no duplicate in lst state list
    let list = LstStatePackedList::of_acc_data(abr.get(*accs.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    list.find_by_mint(lst_mint_acc.key())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::DuplicateLst))?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    // Create pool reserves and protocol fee accumulator ATAs if they do not exist
    if abr.get(*accs.pool_reserves()).data().is_empty() {
        let create_accs = NewCreateIxAccsBuilder::start()
            .with_funding(*accs.payer())
            .with_ata(*accs.pool_reserves())
            .with_wallet(*accs.pool_state())
            .with_mint(*accs.lst_mint())
            .with_sys_prog(*accs.system_program())
            .with_token_prog(*accs.lst_token_program())
            .build();

        cpi.invoke_fwd(abr, &ATOKEN_ID, CreateIxData::as_buf(), create_accs.0)?;
    }

    if abr.get(*accs.protocol_fee_accumulator()).data().is_empty() {
        let create_accs = NewCreateIxAccsBuilder::start()
            .with_funding(*accs.payer())
            .with_ata(*accs.protocol_fee_accumulator())
            .with_wallet(*accs.protocol_fee_accumulator_auth())
            .with_mint(*accs.lst_mint())
            .with_sys_prog(*accs.system_program())
            .with_token_prog(*accs.lst_token_program())
            .build();

        cpi.invoke_fwd(abr, &ATOKEN_ID, CreateIxData::as_buf(), create_accs.0)?;
    }

    // Realloc lst state list
    let lst_state_list_acc = abr.get_mut(*accs.lst_state_list());
    let old_acc_len = lst_state_list_acc.data_len();
    lst_state_list_acc.grow_by(size_of::<LstState>(), false)?;

    let new_acc_len = lst_state_list_acc.data_len();

    let lamports_shortfall = Rent::get()?
        .min_balance(new_acc_len)
        .saturating_sub(Rent::get()?.min_balance(old_acc_len));

    if lamports_shortfall > 0 {
        cpi.invoke_fwd(
            abr,
            &SYS_PROG_ID,
            TransferIxData::new(lamports_shortfall).as_buf(),
            NewTransferIxAccsBuilder::start()
                .with_from(*accs.payer())
                .with_to(*accs.lst_state_list())
                .build()
                .0,
        )?;
    }

    // Add lst state to lst state list
    let sol_value_calculator = *abr.get(*accs.sol_value_calculator()).key();

    let list = LstStatePackedListMut::of_acc_data(abr.get_mut(*accs.lst_state_list()).data_mut())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;
    let new_lst_state_packed = list
        .0
        .last_mut()
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    // safety: account data is 8-byte aligned
    let new_lst_state = unsafe { new_lst_state_packed.as_lst_state_mut() };

    *new_lst_state = LstState {
        is_input_disabled: 0,
        pool_reserves_bump,
        protocol_fee_accumulator_bump,
        padding: [0u8; 5],
        sol_value: 0,
        mint,
        sol_value_calculator,
    };

    Ok(())
}

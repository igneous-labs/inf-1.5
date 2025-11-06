use crate::{
    utils::pay_for_rent_exempt_shortfall,
    verify::{
        verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers,
        verify_sol_value_calculator_is_program, verify_tokenkeg_or_22_mint,
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
    pda_onchain::{find_pool_reserves, find_protocol_fee_accumulator, LST_STATE_LIST_SIGNER},
    program_err::Inf1CtlCustomProgErr,
    typedefs::lst_state::{LstState, LstStatePacked},
    ID,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_SEEDS, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_ata_jiminy::sanctum_ata_core::instructions::create::{
    CreateIdempotentIxData, NewCreateIxAccsBuilder,
};
use sanctum_system_jiminy::{
    instructions::assign::assign_invoke_signed,
    sanctum_system_core::instructions::{
        assign::NewAssignIxAccsBuilder, transfer::NewTransferIxAccsBuilder,
    },
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
        find_pool_reserves(token_prog, &mint).ok_or(INVALID_SEEDS)?;
    let (expected_protocol_fee_accumulator, protocol_fee_accumulator_bump) =
        find_protocol_fee_accumulator(token_prog, &mint).ok_or(INVALID_SEEDS)?;

    let expected_pks = NewAddLstIxAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_lst_mint(lst_mint_acc.key())
        .with_pool_reserves(&expected_pool_reserves)
        .with_protocol_fee_accumulator(&expected_protocol_fee_accumulator)
        .with_protocol_fee_accumulator_auth(&PROTOCOL_FEE_ID)
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_associated_token_program(&ATOKEN_ID)
        .with_system_program(&SYS_PROG_ID)
        .with_lst_token_program(token_prog)
        // Free account - payer can be any account with sufficient lamports for ATA rent
        .with_payer(abr.get(*accs.payer()).key())
        // Free account - admin can specify any sol value calculator program
        .with_sol_value_calculator(abr.get(*accs.sol_value_calculator()).key())
        .build();

    verify_pks(abr, &accs.0, &expected_pks.0)?;
    verify_signers(abr, &accs.0, &ADD_LST_IX_IS_SIGNER.0)?;

    verify_tokenkeg_or_22_mint(lst_mint_acc)?;
    verify_sol_value_calculator_is_program(abr.get(*accs.sol_value_calculator()))?;

    // Verify no duplicate in lst state list
    let lst_state_list_acc = abr.get(*accs.lst_state_list());
    let list = LstStatePackedList::of_acc_data(lst_state_list_acc.data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;

    if list.find_by_mint(lst_mint_acc.key()).is_some() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::DuplicateLst).into());
    }

    verify_not_rebalancing_and_not_disabled(pool)?;

    let is_lsl_uninitialized = lst_state_list_acc.data().is_empty();

    // Create pool reserves and protocol fee accumulator ATAs if they do not exist
    [
        (*accs.pool_reserves(), *accs.pool_state()),
        (
            *accs.protocol_fee_accumulator(),
            *accs.protocol_fee_accumulator_auth(),
        ),
    ]
    .into_iter()
    .try_for_each(|(ata, wallet)| -> Result<(), ProgramError> {
        let create_accs = NewCreateIxAccsBuilder::start()
            .with_funding(*accs.payer())
            .with_ata(ata)
            .with_wallet(wallet)
            .with_mint(*accs.lst_mint())
            .with_sys_prog(*accs.system_program())
            .with_token_prog(*accs.lst_token_program())
            .build();

        cpi.invoke_fwd(
            abr,
            &ATOKEN_ID,
            CreateIdempotentIxData::as_buf(),
            create_accs.0,
        )?;

        Ok(())
    })?;

    // Realloc lst state list
    if is_lsl_uninitialized {
        assign_invoke_signed(
            abr,
            cpi,
            NewAssignIxAccsBuilder::start()
                .with_assign(*accs.lst_state_list())
                .build(),
            &ID,
            &[LST_STATE_LIST_SIGNER],
        )?;
    }

    let lst_state_list_acc = abr.get_mut(*accs.lst_state_list());
    lst_state_list_acc.grow_by(size_of::<LstStatePacked>(), false)?;

    pay_for_rent_exempt_shortfall(
        abr,
        cpi,
        &NewTransferIxAccsBuilder::start()
            .with_from(*accs.payer())
            .with_to(*accs.lst_state_list())
            .build(),
        &Rent::get()?,
    )?;

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

use crate::{verify::verify_pks, Accounts};
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    err::Inf1CtlErr,
    instructions::lst::add::{AddLstIxAccs, NewAddLstIxAccsBuilder},
    keys::{ATOKEN_ID, LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID, SYS_PROG_ID},
    pda::{const_find_pool_reserves, const_find_protocol_fee_accumulator},
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS};

#[inline]
pub fn process_add_lst(accounts: &mut Accounts<'_>) -> Result<(), ProgramError> {
    let accs = accounts
        .as_slice()
        .first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = AddLstIxAccs(*accs);

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(*accs.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let lst_mint_acc = accounts.get(*accs.lst_mint());
    let token_prog = lst_mint_acc.owner();

    let (pool_reserves, _pool_reserves_bump) =
        const_find_pool_reserves(token_prog, lst_mint_acc.key());
    let (protocol_fee_accumulator, _protocol_fee_accumulator_bump) =
        const_find_protocol_fee_accumulator(token_prog, lst_mint_acc.key());

    let expected_pks = NewAddLstIxAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_payer(accounts.get(*accs.payer()).key())
        .with_lst_mint(lst_mint_acc.key())
        .with_pool_reserves(&pool_reserves)
        .with_protocol_fee_accumulator(&protocol_fee_accumulator)
        .with_protocol_fee_accumulator_auth(&PROTOCOL_FEE_ID)
        .with_sol_value_calculator(accounts.get(*accs.sol_value_calculator()).key())
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_associated_token_program(&ATOKEN_ID)
        .with_system_program(&SYS_PROG_ID)
        .with_lst_token_program(token_prog)
        .build();

    verify_pks(accounts, &accs.0, &expected_pks.0)?;
    // verify_signers(accounts, &accs.0, &ADD_LST_IX_IS_SIGNER.0)?;

    Ok(())
}

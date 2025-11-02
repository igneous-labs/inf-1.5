use inf1_ctl_jiminy::{
    accounts::lst_state_list::LstStatePackedList,
    err::Inf1CtlErr,
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
    typedefs::lst_state::{LstState, LstStatePacked},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

pub fn get_lst_state_data<'a>(
    abr: &'a Abr,
    list: &'a LstStatePackedList,
    idx: usize,
    lst_token_program: AccountHandle<'a>,
) -> Result<(&'a LstState, [u8; 32]), ProgramError> {
    let lst_state: &LstStatePacked = list
        .0
        .get(idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let lst_state = unsafe { lst_state.as_lst_state() };

    let expected_reserves = create_raw_pool_reserves_addr(
        abr.get(lst_token_program).key(),
        &lst_state.mint,
        &lst_state.pool_reserves_bump,
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    Ok((lst_state, expected_reserves))
}

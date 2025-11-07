use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked_mut, lst_state_list_get_mut},
    instructions::admin::lst_input::SetLstInputIxAccs,
    typedefs::{lst_state::LstState, u8bool::U8BoolMut},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

#[inline]
pub fn process_enable_lst_input(
    abr: &mut Abr,
    accs: &SetLstInputIxAccs<AccountHandle>,
    idx: usize,
) -> Result<(), ProgramError> {
    let LstState {
        is_input_disabled, ..
    } = lst_state_list_checked_mut(abr.get_mut(*accs.lst_state_list()))
        .and_then(|l| lst_state_list_get_mut(l, idx))?;
    U8BoolMut(is_input_disabled).set_false();
    Ok(())
}

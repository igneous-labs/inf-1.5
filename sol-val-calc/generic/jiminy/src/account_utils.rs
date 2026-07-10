use inf1_svc_generic::{
    accounts::{external::parse_bpf_loader_v3_programdata_meta, state::State},
    errs::GenSvcErr,
    keys::GLOBAL_CONST_KEYS_OWNED,
};
use jiminy_account::Account;

use crate::program_err::GenSvcProgErr;

#[inline]
pub fn state_checked(acc: &Account) -> Result<&State, GenSvcProgErr> {
    // safety: account data is 8-byte aligned
    unsafe { State::of_acc_data(acc.data()) }
        .ok_or(GenSvcProgErr(GenSvcErr::InvalidCalculatorStateData))
}

#[inline]
pub fn state_checked_mut(acc: &mut Account) -> Result<&mut State, GenSvcProgErr> {
    // safety: account data is 8-byte aligned
    unsafe { State::of_acc_data_mut(acc.data_mut()) }
        .ok_or(GenSvcProgErr(GenSvcErr::InvalidCalculatorStateData))
}

#[inline]
pub fn bpf_loader_v3_programdata_checked(
    acc: &Account,
) -> Result<(u64, Option<&[u8; 32]>), GenSvcProgErr> {
    (acc.owner() == GLOBAL_CONST_KEYS_OWNED.bpf_loader_v3())
        .then_some(())
        .and_then(|()| acc.data().first_chunk())
        .and_then(parse_bpf_loader_v3_programdata_meta)
        .ok_or(GenSvcProgErr(GenSvcErr::InvalidStakePoolProgramData))
}

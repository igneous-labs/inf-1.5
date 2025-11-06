use inf1_ctl_core::{
    accounts::{
        disable_pool_authority_list::{DisablePoolAuthorityList, DisablePoolAuthorityListMut},
        lst_state_list::{LstStateList, LstStateListMut},
        packed_list::{PackedList, PackedListMut},
        pool_state::PoolState,
    },
    err::Inf1CtlErr,
    typedefs::lst_state::LstState,
};
use jiminy_cpi::account::Account;

use crate::program_err::Inf1CtlCustomProgErr;

const _ACC_DATA_ALIGN: usize = 8;

const _POOL_STATE_ALIGN_CHECK: () = assert!(core::mem::align_of::<PoolState>() <= _ACC_DATA_ALIGN);

#[inline]
pub fn pool_state_checked(acc: &Account) -> Result<&PoolState, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe { PoolState::of_acc_data(acc.data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))
}

#[inline]
pub fn pool_state_checked_mut(acc: &mut Account) -> Result<&mut PoolState, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe { PoolState::of_acc_data_mut(acc.data_mut()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))
}

/// # Safety
/// - `T` must have align <= align of account data (8)
#[inline]
unsafe fn packed_list_checked<'a, T>(
    acc: &'a Account,
    err: Inf1CtlCustomProgErr,
) -> Result<PackedList<'a, T>, Inf1CtlCustomProgErr> {
    PackedList::of_acc_data_unsafe(acc.data()).ok_or(err)
}

/// # Safety
/// - same conditions as [`packed_list_checked`] apply
#[inline]
unsafe fn packed_list_checked_mut<'a, T>(
    acc: &'a mut Account,
    err: Inf1CtlCustomProgErr,
) -> Result<PackedListMut<'a, T>, Inf1CtlCustomProgErr> {
    PackedListMut::of_acc_data_unsafe(acc.data_mut()).ok_or(err)
}

const _LST_STATE_ALIGN_CHECK: () = assert!(core::mem::align_of::<LstState>() <= _ACC_DATA_ALIGN);

#[inline]
pub fn lst_state_list_checked(acc: &Account) -> Result<LstStateList<'_>, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe {
        packed_list_checked(
            acc,
            Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData),
        )
    }
}

#[inline]
pub fn lst_state_list_checked_mut(
    acc: &mut Account,
) -> Result<LstStateListMut<'_>, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe {
        packed_list_checked_mut(
            acc,
            Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData),
        )
    }
}

const _DISABLE_POOL_AUTH_LIST_ALIGN_CHECK: () =
    assert!(core::mem::align_of::<LstState>() <= _ACC_DATA_ALIGN);

#[inline]
pub fn disable_pool_auth_list_checked(
    acc: &Account,
) -> Result<DisablePoolAuthorityList<'_>, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe {
        packed_list_checked(
            acc,
            Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData),
        )
    }
}

#[inline]
pub fn disable_pool_auth_list_checked_mut(
    acc: &mut Account,
) -> Result<DisablePoolAuthorityListMut<'_>, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe {
        packed_list_checked_mut(
            acc,
            Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData),
        )
    }
}

use jiminy_account::Account;
use jiminy_program_error::{ProgramError, INVALID_ACCOUNT_DATA};
use sanctum_sols_core::{
    accounts::{PoolV1Acc, PoolV1LamportVals, ProtocolV1Acc, RebalAuxData},
    err::SanctumSolsErr,
    typedefs::{
        HoldingFlags, HoldingV1Entry, PkedList, PkedListMut, PoolFlags, ProtocolFlags, Vers,
    },
};

use crate::onchain::err::SanctumSolsProgErr;

const IAD_ERR: ProgramError = ProgramError(INVALID_ACCOUNT_DATA);

#[inline]
const fn verify_pool_v1(p: &PoolV1Acc) -> Result<(), ProgramError> {
    // try_of_ref takes 6 additional CUs compared to unsafe { of_ref }
    let f = match PoolFlags::try_of_ref(&p.flags) {
        None => return Err(IAD_ERR),
        Some(f) => f,
    };
    if !matches!(f.vers(), Vers::V1) {
        return Err(IAD_ERR);
    }
    Ok(())
}

#[inline]
const fn try_rebal_aux(suf: &[u8]) -> Result<Option<&RebalAuxData>, ProgramError> {
    if suf.is_empty() {
        Ok(None)
    } else {
        // safety: since both acc data and PoolV1Acc are 8-byte aligned,
        // suf is guaranteed to be 8-byte aligned
        match unsafe { RebalAuxData::of_acc_data(suf) } {
            Some(x) => Ok(Some(x)),
            None => Err(IAD_ERR),
        }
    }
}

#[inline]
const fn try_rebal_aux_mut(suf: &mut [u8]) -> Result<Option<&mut RebalAuxData>, ProgramError> {
    if suf.is_empty() {
        Ok(None)
    } else {
        // safety: since both acc data and PoolV1Acc are 8-byte aligned,
        // suf is guaranteed to be 8-byte aligned
        match unsafe { RebalAuxData::of_acc_data_mut(suf) } {
            Some(x) => Ok(Some(x)),
            None => Err(IAD_ERR),
        }
    }
}

/// Errors with [`INVALID_ACCOUNT_DATA`] if
/// - data is not of right size
/// - vers is not correct
#[inline]
pub fn try_pool_v1(acc: &Account) -> Result<(&PoolV1Acc, Option<&RebalAuxData>), ProgramError> {
    let (pool, suf) = acc.data().split_first_chunk().ok_or(IAD_ERR)?;
    // safety: account data guaranteed to be 8-byte aligned
    let pool = unsafe { PoolV1Acc::of_acc_data_arr(pool) };
    let rebal_aux = try_rebal_aux(suf)?;
    verify_pool_v1(pool)?;
    Ok((pool, rebal_aux))
}

#[inline]
pub fn try_non_rebalancing_pool_v1(acc: &Account) -> Result<&PoolV1Acc, ProgramError> {
    let (pool, rebal_aux) = try_pool_v1(acc)?;
    match rebal_aux {
        None => Ok(pool),
        Some(_) => Err(SanctumSolsProgErr(SanctumSolsErr::PoolRebalancing).into()),
    }
}

#[inline]
pub fn try_rebalancing_pool_v1(acc: &Account) -> Result<(&PoolV1Acc, &RebalAuxData), ProgramError> {
    let (pool, rebal_aux) = try_pool_v1(acc)?;
    let rebal_aux = rebal_aux.ok_or(SanctumSolsProgErr(SanctumSolsErr::PoolNotRebalancing))?;
    Ok((pool, rebal_aux))
}

/// See [`try_pool_v1`]
#[inline]
pub fn try_pool_v1_mut(
    acc: &mut Account,
) -> Result<(&mut PoolV1Acc, Option<&mut RebalAuxData>), ProgramError> {
    let (pool, suf) = acc.data_mut().split_first_chunk_mut().ok_or(IAD_ERR)?;
    // safety: account data guaranteed to be 8-byte aligned
    let pool = unsafe { PoolV1Acc::of_acc_data_arr_mut(pool) };
    let rebal_aux = try_rebal_aux_mut(suf)?;
    verify_pool_v1(pool)?;
    Ok((pool, rebal_aux))
}

/// [`PoolV1LamportVals::dep_due_checked`] but with err converted for easier interop
/// with [`jiminy_program_error::ProgramError`]
#[inline]
pub const fn dep_due_checked(
    vals: &PoolV1LamportVals,
    pool_acc_lamports: u64,
) -> Result<u64, SanctumSolsProgErr> {
    match vals.dep_due_checked(pool_acc_lamports) {
        None => Err(SanctumSolsProgErr(SanctumSolsErr::Math)),
        Some(x) => Ok(x),
    }
}

#[inline]
const fn verify_holding_v1_list(v: &[HoldingV1Entry]) -> Result<(), ProgramError> {
    let h = match v.first() {
        None => return Ok(()),
        Some(h) => h,
    };
    let f = match HoldingFlags::try_of_ref(&h.flags) {
        None => return Err(IAD_ERR),
        Some(f) => f,
    };
    match f.vers() {
        Vers::V1 => Ok(()),
        _ => Err(IAD_ERR),
    }
}

#[inline]
pub fn try_portfolio_v1(acc: &Account) -> Result<&[HoldingV1Entry], ProgramError> {
    // safety: account data guaranteed to be 8-byte aligned
    let PkedList(v) = unsafe { PkedList::of_acc_data_unsafe(acc.data()) }.ok_or(IAD_ERR)?;
    verify_holding_v1_list(v)?;
    Ok(v)
}

#[inline]
pub fn try_portfolio_v1_mut(acc: &mut Account) -> Result<&mut [HoldingV1Entry], ProgramError> {
    // safety: account data guaranteed to be 8-byte aligned
    let PkedListMut(v) =
        unsafe { PkedListMut::of_acc_data_unsafe(acc.data_mut()) }.ok_or(IAD_ERR)?;
    verify_holding_v1_list(v)?;
    Ok(v)
}

#[inline]
const fn verify_protocol_v1(p: &ProtocolV1Acc) -> Result<(), ProgramError> {
    // try_of_ref takes 6 additional CUs compared to unsafe { of_ref }
    let f = match ProtocolFlags::try_of_ref(&p.flags) {
        None => return Err(IAD_ERR),
        Some(f) => f,
    };
    if !matches!(f.vers(), Vers::V1) {
        return Err(IAD_ERR);
    }
    Ok(())
}

#[inline]
pub fn try_protocol_v1(acc: &Account) -> Result<(&ProtocolV1Acc, &[[u8; 32]]), ProgramError> {
    // safety: account data guaranteed to be 8-byte aligned
    let (p, l) = unsafe { ProtocolV1Acc::of_acc_data_full(acc.data()) }.ok_or(IAD_ERR)?;
    verify_protocol_v1(p)?;
    Ok((p, l))
}

#[inline]
pub fn try_protocol_v1_mut(
    acc: &mut Account,
) -> Result<(&mut ProtocolV1Acc, &mut [[u8; 32]]), ProgramError> {
    // safety: account data guaranteed to be 8-byte aligned
    let (p, l) = unsafe { ProtocolV1Acc::of_acc_data_full_mut(acc.data_mut()) }.ok_or(IAD_ERR)?;
    verify_protocol_v1(p)?;
    Ok((p, l))
}
